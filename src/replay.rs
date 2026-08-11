use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::error::AppError;
use crate::project::Project;
use crate::render::checksum_bytes;
use crate::simulation::World;

pub const REPLAY_SCHEMA_V1: &str = "aetherion.replay/v1";
pub const REPLAY_SCHEMA: &str = "aetherion.replay/v2";
pub const EVENTS_SCHEMA: &str = "aetherion.events/v1";
const MAX_REPLAY_TICKS: u64 = 1_000_000;
const MAX_REPLAY_ITEMS: usize = 10_000;
const MAX_REPLAY_BYTES: u64 = 4_194_304;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Replay {
    pub schema: String,
    pub project: ReplayProject,
    pub target_tick: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_interval: Option<u64>,
    pub events: Vec<InputEvent>,
    pub checksums: Vec<ExpectedChecksum>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayProject {
    pub name: String,
    pub source_checksum: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EventFile {
    pub schema: String,
    pub events: Vec<InputEvent>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct InputEvent {
    pub tick: u64,
    pub sequence: u64,
    pub entity_id: u64,
    #[serde(flatten)]
    pub command: InputCommand,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum InputCommand {
    SetVelocity { x: i64, y: i64 },
    Impulse { x: i64, y: i64 },
    Translate { x: i64, y: i64 },
    Stop,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedChecksum {
    pub tick: u64,
    pub checksum: u64,
}

#[derive(Debug, Serialize)]
pub struct ReplayReport {
    pub schema: &'static str,
    pub status: &'static str,
    pub replay: String,
    pub target_tick: u64,
    pub verified_checkpoints: usize,
    pub checksum: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub divergence: Option<Divergence>,
}

#[derive(Debug, Serialize)]
pub struct Divergence {
    pub tick: u64,
    pub expected: u64,
    pub actual: u64,
}

pub fn project_fingerprint(project: &Project) -> Result<u64> {
    let canonical = serde_json::to_vec(project)
        .map_err(|error| format!("empreinte du projet impossible: {error}"))?;
    Ok(checksum_bytes(&canonical))
}

pub fn load_events(path: &Path) -> Result<Vec<InputEvent>> {
    let bytes =
        fs::read(path).map_err(|error| format!("lecture de {}: {error}", path.display()))?;
    let file: EventFile = serde_json::from_slice(&bytes)
        .map_err(|error| format!("événements invalides dans {}: {error}", path.display()))?;
    if file.schema != EVENTS_SCHEMA {
        return Err(format!("schéma d'événements non supporté: {}", file.schema).into());
    }
    Ok(file.events)
}

pub fn create(project: Project, target_tick: u64, events: Vec<InputEvent>) -> Result<Replay> {
    create_with_interval(project, target_tick, events, 1)
}

pub fn create_with_interval(
    project: Project,
    target_tick: u64,
    events: Vec<InputEvent>,
    interval: u64,
) -> Result<Replay> {
    if interval == 0 {
        return Err("checkpoint_interval doit être supérieur à 0".into());
    }
    if target_tick > MAX_REPLAY_TICKS || events.len() > MAX_REPLAY_ITEMS {
        return Err("le replay dépasse les limites de ressources".into());
    }
    validate_events(&events, target_tick, &project)?;
    let fingerprint = project_fingerprint(&project)?;
    let project_name = project.project.name.clone();
    let mut world = World::from_project(project);
    let capacity = target_tick / interval + 2;
    let mut checksums = Vec::with_capacity(capacity as usize);
    checksums.push(ExpectedChecksum {
        tick: 0,
        checksum: world.checksum(),
    });
    run_events(&mut world, target_tick, &events, |world| {
        if world.tick % interval == 0 || world.tick == target_tick {
            checksums.push(ExpectedChecksum {
                tick: world.tick,
                checksum: world.checksum(),
            });
        }
        Ok(())
    })?;
    Ok(Replay {
        schema: REPLAY_SCHEMA.into(),
        project: ReplayProject {
            name: project_name,
            source_checksum: fingerprint,
        },
        target_tick,
        checkpoint_interval: Some(interval),
        events,
        checksums,
    })
}

pub fn save(replay: &Replay, output: &Path) -> Result<()> {
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| format!("création de {}: {error}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(replay)
        .map_err(|error| format!("sérialisation du replay: {error}"))?;
    fs::write(output, format!("{json}\n"))
        .map_err(|error| format!("écriture de {}: {error}", output.display()).into())
}

pub fn load(path: &Path) -> Result<Replay> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("lecture de {}: {error}", path.display()))?;
    if metadata.len() > MAX_REPLAY_BYTES {
        return Err("le fichier replay dépasse la limite de 4 MiB".into());
    }
    let bytes =
        fs::read(path).map_err(|error| format!("lecture de {}: {error}", path.display()))?;
    let replay: Replay = serde_json::from_slice(&bytes)
        .map_err(|error| format!("replay invalide dans {}: {error}", path.display()))?;
    if replay.schema != REPLAY_SCHEMA && replay.schema != REPLAY_SCHEMA_V1 {
        return Err(format!("schéma de replay non supporté: {}", replay.schema).into());
    }
    Ok(replay)
}

pub fn play(project: Project, replay: &Replay, replay_name: &str) -> Result<ReplayReport> {
    validate_events(&replay.events, replay.target_tick, &project)?;
    validate_checksums(replay)?;
    let fingerprint = project_fingerprint(&project)?;
    if fingerprint != replay.project.source_checksum || project.project.name != replay.project.name
    {
        return Err("le projet ne correspond pas à l'empreinte du replay".into());
    }
    let mut world = World::from_project(project);
    verify_checkpoint(&world, &replay.checksums[0], replay_name, 0)?;
    let mut checkpoint_index = 1;
    run_events(&mut world, replay.target_tick, &replay.events, |world| {
        if checkpoint_index < replay.checksums.len()
            && replay.checksums[checkpoint_index].tick == world.tick
        {
            verify_checkpoint(
                world,
                &replay.checksums[checkpoint_index],
                replay_name,
                checkpoint_index,
            )?;
            checkpoint_index += 1;
        }
        Ok(())
    })?;
    Ok(ReplayReport {
        schema: "aetherion.replay-report/v1",
        status: "identical",
        replay: replay_name.into(),
        target_tick: replay.target_tick,
        verified_checkpoints: replay.checksums.len(),
        checksum: world.checksum(),
        divergence: None,
    })
}

fn verify_checkpoint(
    world: &World,
    expected: &ExpectedChecksum,
    replay_name: &str,
    verified: usize,
) -> Result<()> {
    let actual = world.checksum();
    if actual == expected.checksum {
        return Ok(());
    }
    let report = ReplayReport {
        schema: "aetherion.replay-report/v1",
        status: "diverged",
        replay: replay_name.into(),
        target_tick: world.tick,
        verified_checkpoints: verified,
        checksum: actual,
        divergence: Some(Divergence {
            tick: world.tick,
            expected: expected.checksum,
            actual,
        }),
    };
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("sérialisation du rapport: {error}"))?;
    Err(AppError::outcome(
        format!("divergence de checksum au tick {}", world.tick),
        3,
        json,
    ))
}

fn validate_checksums(replay: &Replay) -> Result<()> {
    if replay.target_tick > MAX_REPLAY_TICKS
        || replay.events.len() > MAX_REPLAY_ITEMS
        || replay.checksums.len() > MAX_REPLAY_TICKS as usize + 1
    {
        return Err("le replay dépasse les limites de ressources".into());
    }
    if replay.checksums.is_empty()
        || replay.checksums[0].tick != 0
        || replay
            .checksums
            .last()
            .is_none_or(|value| value.tick != replay.target_tick)
    {
        return Err("les checkpoints doivent inclure le tick 0 et le tick final".into());
    }
    if replay.schema == REPLAY_SCHEMA_V1 {
        if replay.checksums.len() != replay.target_tick as usize + 1
            || replay
                .checksums
                .iter()
                .enumerate()
                .any(|(index, value)| value.tick != index as u64)
        {
            return Err("les checksums v1 doivent être ordonnés, uniques et continus".into());
        }
    } else {
        let interval = replay
            .checkpoint_interval
            .ok_or("checkpoint_interval absent du replay v2")?;
        if interval == 0 {
            return Err("checkpoint_interval doit être supérieur à 0".into());
        }
        for (index, pair) in replay.checksums.windows(2).enumerate() {
            if pair[0].tick >= pair[1].tick
                || (pair[1].tick != replay.target_tick && pair[1].tick % interval != 0)
                || (index > 0 && pair[0].tick % interval != 0)
            {
                return Err("checkpoints v2 invalides ou désordonnés".into());
            }
        }
    }
    Ok(())
}

pub fn validate_events(events: &[InputEvent], target_tick: u64, project: &Project) -> Result<()> {
    let ids: HashSet<u64> = project.entities.iter().map(|entity| entity.id).collect();
    let mut previous = None;
    for event in events {
        if event.tick >= target_tick {
            return Err(
                format!("tick d'événement {} hors cible {}", event.tick, target_tick).into(),
            );
        }
        if !ids.contains(&event.entity_id) {
            return Err(format!("entité inconnue dans les événements: {}", event.entity_id).into());
        }
        let key = (event.tick, event.sequence);
        if previous.is_some_and(|value| value >= key) {
            return Err(
                "les événements doivent être triés par (tick, sequence), sans doublon".into(),
            );
        }
        previous = Some(key);
    }
    Ok(())
}

fn run_events<F>(
    world: &mut World,
    target_tick: u64,
    events: &[InputEvent],
    mut after_step: F,
) -> Result<()>
where
    F: FnMut(&World) -> Result<()>,
{
    let mut index = 0;
    while world.tick < target_tick {
        let start = index;
        while index < events.len() && events[index].tick == world.tick {
            index += 1;
        }
        world.step_with_events(&events[start..index])?;
        after_step(world)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> Project {
        toml::from_str(Project::example()).unwrap()
    }

    #[test]
    fn event_order_is_strict() {
        let events = vec![
            InputEvent {
                tick: 1,
                sequence: 2,
                entity_id: 1,
                command: InputCommand::Stop,
            },
            InputEvent {
                tick: 1,
                sequence: 1,
                entity_id: 1,
                command: InputCommand::Stop,
            },
        ];
        assert!(validate_events(&events, 3, &project()).is_err());
    }

    #[test]
    fn identical_replay_is_verified() {
        let events = vec![InputEvent {
            tick: 2,
            sequence: 0,
            entity_id: 1,
            command: InputCommand::SetVelocity { x: 3, y: -1 },
        }];
        let replay = create(project(), 6, events).unwrap();
        let report = play(project(), &replay, "memory").unwrap();
        assert_eq!(report.status, "identical");
    }

    #[test]
    fn changed_checksum_diverges() {
        let mut replay = create(project(), 2, Vec::new()).unwrap();
        replay.checksums[1].checksum ^= 1;
        let error = play(project(), &replay, "memory").unwrap_err();
        assert_eq!(error.exit_code, 3);
        assert!(error.json.unwrap().contains("diverged"));
    }

    #[test]
    fn interval_one_and_spaced_checkpoints_are_exact() {
        let every_tick = create_with_interval(project(), 10, Vec::new(), 1).unwrap();
        assert_eq!(
            every_tick
                .checksums
                .iter()
                .map(|value| value.tick)
                .collect::<Vec<_>>(),
            (0..=10).collect::<Vec<_>>()
        );
        let spaced = create_with_interval(project(), 10, Vec::new(), 4).unwrap();
        assert_eq!(
            spaced
                .checksums
                .iter()
                .map(|value| value.tick)
                .collect::<Vec<_>>(),
            vec![0, 4, 8, 10]
        );
        assert_eq!(
            play(project(), &spaced, "memory")
                .unwrap()
                .verified_checkpoints,
            4
        );
    }

    #[test]
    fn existing_v1_fixture_is_read_and_verified() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("demo/demo.replay.json");
        let replay = load(&path).unwrap();
        assert_eq!(replay.schema, REPLAY_SCHEMA_V1);
        let demo = Project::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("demo")).unwrap();
        assert_eq!(play(demo, &replay, "demo-v1").unwrap().status, "identical");
    }
}
