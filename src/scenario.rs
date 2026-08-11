use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::error::AppError;
use crate::project::{Project, RenderConfig};
use crate::render::{checksum_bytes, render};
use crate::replay::{InputEvent, project_fingerprint, validate_events};
use crate::simulation::{Vec2, World};

pub const SCENARIO_SCHEMA: &str = "aetherion.scenario/v1";
pub const REPORT_SCHEMA: &str = "aetherion.scenario-report/v1";
pub const AUDIT_SCHEMA: &str = "aetherion.audit/v1";
const HARD_MAX_INPUT_BYTES: u64 = 1_048_576;
const HARD_MAX_OUTPUT_BYTES: u64 = 4_194_304;
const HARD_MAX_TICKS: u64 = 1_000_000;
const HARD_MAX_ITEMS: usize = 10_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub schema: String,
    pub project: ScenarioProject,
    pub max_ticks: u64,
    #[serde(default)]
    pub events: Vec<InputEvent>,
    #[serde(default)]
    pub assertions: Vec<Assertion>,
    pub budgets: Budgets,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioProject {
    pub name: String,
    #[serde(default)]
    pub source_checksum: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Budgets {
    pub max_ticks: u64,
    pub max_events: usize,
    pub max_assertions: usize,
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
    #[serde(default)]
    pub advisory_timeout_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Assertion {
    pub id: String,
    #[serde(default)]
    pub tick: Option<u64>,
    #[serde(flatten)]
    pub expectation: Expectation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Expectation {
    Checksum { value: u64 },
    EntityPosition { entity_id: u64, x: i64, y: i64 },
    EntityVelocity { entity_id: u64, x: i64, y: i64 },
    EntityCount { value: usize },
    EntityVisible { entity_id: u64, value: bool },
}

#[derive(Clone, Debug, Serialize)]
pub struct ScenarioReport {
    pub schema: &'static str,
    pub status: &'static str,
    pub run_id: String,
    pub project_fingerprint: u64,
    pub scenario_fingerprint: u64,
    pub target_tick: u64,
    pub assertions: Vec<AssertionResult>,
    pub final_state: FinalState,
    pub consumption: Consumption,
    pub failures: Vec<String>,
    pub files: ProducedFiles,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssertionResult {
    pub id: String,
    pub tick: u64,
    pub kind: &'static str,
    pub passed: bool,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FinalState {
    pub tick: u64,
    pub checksum: u64,
    pub entity_count: usize,
    pub entities: Vec<FinalEntity>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FinalEntity {
    pub id: u64,
    pub position: Vec2,
    pub velocity: Vec2,
}

#[derive(Clone, Debug, Serialize)]
pub struct Consumption {
    pub ticks: u64,
    pub events: usize,
    pub assertions: usize,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub limits: Budgets,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ProducedFiles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuditRecord<'a> {
    schema: &'static str,
    run_id: &'a str,
    project_fingerprint: u64,
    scenario_fingerprint: u64,
    command: AuditCommand,
    status: &'a str,
    ticks: u64,
    events: usize,
    assertions: usize,
    files: &'a ProducedFiles,
}

#[derive(Debug, Serialize)]
struct AuditCommand {
    name: &'static str,
    project: String,
    scenario: String,
}

pub fn load(path: &Path) -> Result<(Scenario, Vec<u8>)> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("lecture de {}: {error}", path.display()))?;
    if metadata.len() > HARD_MAX_INPUT_BYTES {
        return Err(budget_error(
            "fichier de scénario supérieur à la limite absolue de 1 MiB",
        ));
    }
    let bytes =
        fs::read(path).map_err(|error| format!("lecture de {}: {error}", path.display()))?;
    let scenario: Scenario = serde_json::from_slice(&bytes)
        .map_err(|error| format!("scénario invalide dans {}: {error}", path.display()))?;
    validate_shape(&scenario, bytes.len() as u64)?;
    Ok((scenario, bytes))
}

pub fn run(
    project_dir: &Path,
    scenario_path: &Path,
    report_path: Option<&Path>,
    audit_path: Option<&Path>,
) -> Result<ScenarioReport> {
    let (scenario, source) = load(scenario_path)?;
    let project = Project::load(project_dir)?;
    let project_hash = project_fingerprint(&project)?;
    if project.project.name != scenario.project.name {
        return Err("le nom du projet ne correspond pas au scénario".into());
    }
    if scenario
        .project
        .source_checksum
        .is_some_and(|expected| expected != project_hash)
    {
        return Err("l'empreinte du projet ne correspond pas au scénario".into());
    }
    validate_events(&scenario.events, scenario.max_ticks, &project)?;

    let scenario_hash = checksum_bytes(&source);
    let run_id = format!("{:016x}", deterministic_run_id(project_hash, scenario_hash));
    let files = ProducedFiles {
        report: report_path.map(normalized_path),
        audit: audit_path.map(normalized_path),
    };
    let render_config = project.render.clone();
    let mut world = World::from_project(project);
    let mut results = Vec::with_capacity(scenario.assertions.len());
    let mut event_index = 0;

    evaluate_at_tick(&scenario, &world, &render_config, &mut results)?;
    while world.tick < scenario.max_ticks {
        let start = event_index;
        while event_index < scenario.events.len() && scenario.events[event_index].tick == world.tick
        {
            event_index += 1;
        }
        world.step_with_events(&scenario.events[start..event_index])?;
        evaluate_at_tick(&scenario, &world, &render_config, &mut results)?;
    }

    let failures: Vec<String> = results
        .iter()
        .filter(|result| !result.passed)
        .map(|result| format!("assertion {} échouée au tick {}", result.id, result.tick))
        .collect();
    let status = if failures.is_empty() { "pass" } else { "fail" };
    let mut report = ScenarioReport {
        schema: REPORT_SCHEMA,
        status,
        run_id,
        project_fingerprint: project_hash,
        scenario_fingerprint: scenario_hash,
        target_tick: scenario.max_ticks,
        assertions: results,
        final_state: FinalState {
            tick: world.tick,
            checksum: world.checksum(),
            entity_count: world.entity_count(),
            entities: world
                .entities()
                .map(|entity| FinalEntity {
                    id: entity.id,
                    position: entity.position,
                    velocity: entity.velocity,
                })
                .collect(),
        },
        consumption: Consumption {
            ticks: scenario.max_ticks,
            events: scenario.events.len(),
            assertions: scenario.assertions.len(),
            input_bytes: source.len() as u64,
            output_bytes: 0,
            limits: scenario.budgets.clone(),
        },
        failures,
        files,
    };
    let first = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("sérialisation du rapport: {error}"))?;
    report.consumption.output_bytes = (first.len() + 1) as u64;
    let json = serde_json::to_vec_pretty(&report)
        .map_err(|error| format!("sérialisation du rapport: {error}"))?;
    if (json.len() + 1) as u64 > scenario.budgets.max_output_bytes {
        return Err(budget_error("le rapport dépasse budgets.max_output_bytes"));
    }
    if let Some(path) = report_path {
        atomic_write(path, &with_newline(&json))?;
    }
    if let Some(path) = audit_path {
        append_audit(path, project_dir, scenario_path, &report)?;
    }
    if report.status == "fail" {
        let printable = String::from_utf8(with_newline(&json))
            .map_err(|error| format!("rapport UTF-8 invalide: {error}"))?;
        return Err(AppError::outcome("des assertions ont échoué", 1, printable));
    }
    Ok(report)
}

fn validate_shape(scenario: &Scenario, input_bytes: u64) -> Result<()> {
    if scenario.schema != SCENARIO_SCHEMA {
        return Err(format!("schéma de scénario non supporté: {}", scenario.schema).into());
    }
    if scenario.project.name.trim().is_empty() {
        return Err("scenario.project.name ne peut pas être vide".into());
    }
    let budgets = &scenario.budgets;
    if budgets.max_ticks > HARD_MAX_TICKS
        || budgets.max_events > HARD_MAX_ITEMS
        || budgets.max_assertions > HARD_MAX_ITEMS
        || budgets.max_input_bytes > HARD_MAX_INPUT_BYTES
        || budgets.max_output_bytes > HARD_MAX_OUTPUT_BYTES
    {
        return Err(budget_error("un budget déclaré dépasse une limite absolue"));
    }
    if scenario.max_ticks > budgets.max_ticks
        || scenario.events.len() > budgets.max_events
        || scenario.assertions.len() > budgets.max_assertions
        || input_bytes > budgets.max_input_bytes
    {
        return Err(budget_error("le scénario dépasse un budget déclaré"));
    }
    if budgets.max_output_bytes == 0 || budgets.max_input_bytes == 0 {
        return Err(budget_error(
            "les budgets d'entrée/sortie doivent être non nuls",
        ));
    }
    let mut ids = std::collections::HashSet::new();
    for assertion in &scenario.assertions {
        if assertion.id.trim().is_empty() || !ids.insert(&assertion.id) {
            return Err("les identifiants d'assertion doivent être non vides et uniques".into());
        }
        if assertion.tick.unwrap_or(scenario.max_ticks) > scenario.max_ticks {
            return Err(format!("tick hors cible pour l'assertion {}", assertion.id).into());
        }
    }
    Ok(())
}

fn evaluate_at_tick(
    scenario: &Scenario,
    world: &World,
    render_config: &RenderConfig,
    output: &mut Vec<AssertionResult>,
) -> Result<()> {
    for assertion in scenario
        .assertions
        .iter()
        .filter(|assertion| assertion.tick.unwrap_or(scenario.max_ticks) == world.tick)
    {
        output.push(evaluate(assertion, world, render_config)?);
    }
    Ok(())
}

fn evaluate(
    assertion: &Assertion,
    world: &World,
    config: &RenderConfig,
) -> Result<AssertionResult> {
    let (kind, expected, actual, reason) = match &assertion.expectation {
        Expectation::Checksum { value } => (
            "checksum",
            serde_json::json!(value),
            serde_json::json!(world.checksum()),
            None,
        ),
        Expectation::EntityCount { value } => (
            "entity_count",
            serde_json::json!(value),
            serde_json::json!(world.entity_count()),
            None,
        ),
        Expectation::EntityPosition { entity_id, x, y } => {
            let entity = world.entities().find(|entity| entity.id == *entity_id);
            (
                "entity_position",
                serde_json::json!({"x":x,"y":y}),
                entity
                    .as_ref()
                    .map(|entity| serde_json::json!(entity.position))
                    .unwrap_or(serde_json::Value::Null),
                entity
                    .is_none()
                    .then(|| format!("entité inconnue: {entity_id}")),
            )
        }
        Expectation::EntityVelocity { entity_id, x, y } => {
            let entity = world.entities().find(|entity| entity.id == *entity_id);
            (
                "entity_velocity",
                serde_json::json!({"x":x,"y":y}),
                entity
                    .as_ref()
                    .map(|entity| serde_json::json!(entity.velocity))
                    .unwrap_or(serde_json::Value::Null),
                entity
                    .is_none()
                    .then(|| format!("entité inconnue: {entity_id}")),
            )
        }
        Expectation::EntityVisible { entity_id, value } => {
            let (_, visible) = render(world, config)?;
            let actual = visible.iter().any(|entity| entity.id == *entity_id);
            (
                "entity_visible",
                serde_json::json!(value),
                serde_json::json!(actual),
                None,
            )
        }
    };
    Ok(AssertionResult {
        id: assertion.id.clone(),
        tick: world.tick,
        kind,
        passed: reason.is_none() && expected == actual,
        expected,
        actual,
        reason,
    })
}

fn deterministic_run_id(project: u64, scenario: u64) -> u64 {
    checksum_bytes(format!("{project:016x}:{scenario:016x}").as_bytes())
}

fn budget_error(message: &str) -> AppError {
    AppError::new(message).with_exit_code(3)
}

fn with_newline(bytes: &[u8]) -> Vec<u8> {
    let mut value = Vec::with_capacity(bytes.len() + 1);
    value.extend_from_slice(bytes);
    value.push(b'\n');
    value
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| format!("création de {}: {error}", parent.display()))?;
    }
    let mut temporary = PathBuf::from(path);
    temporary.set_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temporary, bytes)
        .map_err(|error| format!("écriture de {}: {error}", temporary.display()))?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("remplacement de {}: {error}", path.display()))?;
    }
    fs::rename(&temporary, path)
        .map_err(|error| format!("finalisation de {}: {error}", path.display()).into())
}

fn append_audit(
    path: &Path,
    project_dir: &Path,
    scenario_path: &Path,
    report: &ScenarioReport,
) -> Result<()> {
    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| format!("création de {}: {error}", parent.display()))?;
    }
    let record = AuditRecord {
        schema: AUDIT_SCHEMA,
        run_id: &report.run_id,
        project_fingerprint: report.project_fingerprint,
        scenario_fingerprint: report.scenario_fingerprint,
        command: AuditCommand {
            name: "scenario-run",
            project: normalized_path(project_dir),
            scenario: normalized_path(scenario_path),
        },
        status: report.status,
        ticks: report.consumption.ticks,
        events: report.consumption.events,
        assertions: report.consumption.assertions,
        files: &report.files,
    };
    let line = serde_json::to_vec(&record)
        .map_err(|error| format!("sérialisation de l'audit: {error}"))?;
    if line.len() as u64 + 1 > HARD_MAX_OUTPUT_BYTES {
        return Err(budget_error("entrée d'audit trop grande"));
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("ouverture de {}: {error}", path.display()))?;
    file.write_all(&with_newline(&line))
        .map_err(|error| format!("écriture de {}: {error}", path.display()).into())
}

fn normalized_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal() -> Scenario {
        Scenario {
            schema: SCENARIO_SCHEMA.into(),
            project: ScenarioProject {
                name: "hello-aetherion".into(),
                source_checksum: None,
            },
            max_ticks: 2,
            events: Vec::new(),
            assertions: vec![Assertion {
                id: "position".into(),
                tick: None,
                expectation: Expectation::EntityPosition {
                    entity_id: 1,
                    x: 2,
                    y: 0,
                },
            }],
            budgets: Budgets {
                max_ticks: 2,
                max_events: 0,
                max_assertions: 1,
                max_input_bytes: 4096,
                max_output_bytes: 65536,
                advisory_timeout_ms: Some(1000),
            },
        }
    }

    #[test]
    fn parsing_is_strict_and_versioned() {
        let json = serde_json::to_vec(&minimal()).unwrap();
        let parsed: Scenario = serde_json::from_slice(&json).unwrap();
        validate_shape(&parsed, json.len() as u64).unwrap();
        assert_eq!(parsed.schema, SCENARIO_SCHEMA);
        assert!(serde_json::from_str::<Scenario>(r#"{"schema":"bad","extra":1}"#).is_err());
    }

    #[test]
    fn declared_budget_is_enforced() {
        let mut scenario = minimal();
        scenario.max_ticks = 3;
        let error = validate_shape(&scenario, 100).unwrap_err();
        assert_eq!(error.exit_code, 3);
    }

    #[test]
    fn assertion_reports_expected_and_actual() {
        let project: Project = toml::from_str(Project::example()).unwrap();
        let config = project.render.clone();
        let mut world = World::from_project(project);
        world.run(2).unwrap();
        let result = evaluate(&minimal().assertions[0], &world, &config).unwrap();
        assert!(result.passed);
        assert_eq!(result.actual["x"], 2);
    }
}
