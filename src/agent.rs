use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::Result;
use crate::capture::{self, ImageFormat, ViewsFile};
use crate::diff;
use crate::project::{Project, RenderConfig};
use crate::replay::InputEvent;
use crate::simulation::World;

const REQUEST_SCHEMA: &str = "aetherion.agent-request/v1";
const RESPONSE_SCHEMA: &str = "aetherion.agent-response/v1";
const POLICY_SCHEMA: &str = "aetherion.capability-policy/v1";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub project_read: bool,
    pub world_mutate: bool,
    pub capture: bool,
    pub file_write: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            project_read: true,
            world_mutate: true,
            capture: true,
            file_write: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub max_line_bytes: usize,
    pub max_operations: usize,
    pub max_ticks_per_request: u64,
    pub max_events: usize,
    pub max_captures: usize,
    pub max_output_bytes: usize,
    pub max_audit_bytes: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_line_bytes: 1_048_576,
            max_operations: 64,
            max_ticks_per_request: 10_000,
            max_events: 10_000,
            max_captures: 8,
            max_output_bytes: 4_194_304,
            max_audit_bytes: 4_194_304,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub schema: String,
    pub capabilities: Capabilities,
    pub limits: Limits,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            schema: POLICY_SCHEMA.into(),
            capabilities: Capabilities::default(),
            limits: Limits::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    request_id: String,
    method: String,
    params: Value,
}

#[derive(Debug, Serialize)]
struct ProtocolError {
    code: &'static str,
    message: String,
    retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

impl ProtocolError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: false,
            details: None,
        }
    }
    fn details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

type ProtocolResult<T> = std::result::Result<T, ProtocolError>;

#[derive(Clone)]
struct Session {
    world: World,
    revision: u64,
}

struct Agent {
    project: Project,
    render: RenderConfig,
    root: PathBuf,
    policy: Policy,
    session: Option<Session>,
    audit: Option<PathBuf>,
    next_session: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionRef {
    session_id: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MutateParams {
    session_id: String,
    #[serde(default)]
    ticks: u64,
    #[serde(default)]
    events: Vec<InputEvent>,
    expected_revision: Option<u64>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputParams {
    session_id: String,
    events: Vec<InputEvent>,
    expected_revision: Option<u64>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureParams {
    session_id: String,
    path: String,
    #[serde(default)]
    format: ImageFormat,
    expected_revision: Option<u64>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MultiParams {
    session_id: String,
    output_dir: String,
    views: ViewsFile,
    #[serde(default)]
    dry_run: bool,
    expected_revision: Option<u64>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DiffParams {
    session_id: String,
    snapshot: Value,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Operation {
    method: String,
    params: Value,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TransactionParams {
    session_id: String,
    operations: Vec<Operation>,
    #[serde(default)]
    dry_run: bool,
    expected_revision: Option<u64>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StepOperation {
    #[serde(default)]
    ticks: u64,
    #[serde(default)]
    events: Vec<InputEvent>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputOperation {
    events: Vec<InputEvent>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureOperation {
    path: String,
    #[serde(default)]
    format: ImageFormat,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MultiOperation {
    output_dir: String,
    views: ViewsFile,
}

#[derive(Clone)]
enum PreparedCapture {
    Single {
        target: PathBuf,
        format: ImageFormat,
    },
    Multi {
        target: PathBuf,
        views: ViewsFile,
    },
}

pub fn run(
    project_path: &Path,
    root: &Path,
    policy_path: Option<&Path>,
    audit: Option<&Path>,
) -> Result<()> {
    let project = Project::load(project_path)?;
    let root =
        fs::canonicalize(root).map_err(|e| format!("racine invalide {}: {e}", root.display()))?;
    let policy = load_policy(policy_path)?;
    let audit = audit
        .map(|path| confined_path(&root, path))
        .transpose()
        .map_err(|e| e.message)?;
    let mut agent = Agent {
        render: project.render.clone(),
        project,
        root,
        policy,
        session: None,
        audit,
        next_session: 1,
    };
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    for line in stdin.lock().split(b'\n') {
        let line = line.map_err(|e| format!("lecture stdin: {e}"))?;
        if line.is_empty() {
            continue;
        }
        let response = agent.process_line(&line);
        let mut bytes =
            serde_json::to_vec(&response).map_err(|e| format!("sérialisation de réponse: {e}"))?;
        if bytes.len() > agent.policy.limits.max_output_bytes {
            bytes = serde_json::to_vec(&error_response(
                response
                    .get("request_id")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
                ProtocolError::new("quota_exceeded", "réponse supérieure à max_output_bytes"),
            ))
            .map_err(|e| format!("sérialisation de réponse: {e}"))?;
        }
        output
            .write_all(&bytes)
            .and_then(|_| output.write_all(b"\n"))
            .and_then(|_| output.flush())
            .map_err(|e| format!("écriture stdout: {e}"))?;
    }
    Ok(())
}

fn load_policy(path: Option<&Path>) -> Result<Policy> {
    let Some(path) = path else {
        return Ok(Policy::default());
    };
    let bytes = fs::read(path).map_err(|e| format!("lecture de {}: {e}", path.display()))?;
    let policy: Policy = serde_json::from_slice(&bytes)
        .map_err(|e| format!("politique invalide dans {}: {e}", path.display()))?;
    if policy.schema != POLICY_SCHEMA {
        return Err("schéma de politique incompatible".into());
    }
    if policy.limits.max_line_bytes > 4_194_304
        || policy.limits.max_operations > 1024
        || policy.limits.max_ticks_per_request > 1_000_000
        || policy.limits.max_events > 10_000
        || policy.limits.max_captures > 64
        || policy.limits.max_output_bytes > 16_777_216
        || policy.limits.max_audit_bytes > 16_777_216
    {
        return Err("la politique dépasse les plafonds absolus".into());
    }
    Ok(policy)
}

impl Agent {
    fn process_line(&mut self, line: &[u8]) -> Value {
        if line.len() > self.policy.limits.max_line_bytes {
            return error_response(
                "",
                ProtocolError::new("quota_exceeded", "ligne supérieure à max_line_bytes"),
            );
        }
        let request: Request = match serde_json::from_slice(line) {
            Ok(value) => value,
            Err(error) => {
                return error_response(
                    "",
                    ProtocolError::new(
                        "invalid_request",
                        format!("requête JSON invalide: {error}"),
                    ),
                );
            }
        };
        let id = request.request_id.clone();
        let method = request.method.clone();
        let result = if request.schema != REQUEST_SCHEMA {
            Err(ProtocolError::new(
                "incompatible_version",
                format!("schéma attendu: {REQUEST_SCHEMA}"),
            ))
        } else if request.request_id.is_empty()
            || request.request_id.len() > 128
            || request.method.is_empty()
            || request.method.len() > 64
        {
            Err(ProtocolError::new(
                "invalid_request",
                "request_id ou method invalide",
            ))
        } else {
            self.dispatch(&request.method, request.params)
        };
        let verdict = if result.is_ok() { "ok" } else { "error" };
        self.write_audit(&id, &method, verdict);
        match result {
            Ok(result) => {
                json!({"schema":RESPONSE_SCHEMA,"request_id":id,"status":"ok","result":result})
            }
            Err(error) => error_response(&id, error),
        }
    }

    fn dispatch(&mut self, method: &str, params: Value) -> ProtocolResult<Value> {
        match method {
            "handshake" => {
                parse::<Empty>(params)?;
                Ok(json!({
                    "version":1,"request_schema":REQUEST_SCHEMA,"response_schema":RESPONSE_SCHEMA,
                    "methods":["handshake","session.create","session.close","world.inspect","world.step","world.run","input.apply","capture.create","capture.multi","state.diff","snapshot.diff","transaction.execute"],
                    "capabilities":self.policy.capabilities,"limits":self.policy.limits,"network":false
                }))
            }
            "session.create" => self.session_create(params),
            "session.close" => self.session_close(params),
            "world.inspect" => self.inspect(params),
            "world.step" | "world.run" => self.mutate(params),
            "input.apply" => self.input(params),
            "capture.create" => self.capture(params),
            "capture.multi" => self.capture_multi(params),
            "state.diff" | "snapshot.diff" => self.state_diff(params),
            "transaction.execute" => self.transaction(params),
            _ => Err(ProtocolError::new(
                "method_not_found",
                format!("méthode inconnue: {method}"),
            )),
        }
    }

    fn session_create(&mut self, params: Value) -> ProtocolResult<Value> {
        parse::<Empty>(params)?;
        require(self.policy.capabilities.project_read, "project_read")?;
        if self.session.is_some() {
            return Err(ProtocolError::new(
                "quota_exceeded",
                "une seule session est autorisée",
            ));
        }
        let id = format!("session-{}", self.next_session);
        self.next_session += 1;
        let world = World::from_project(self.project.clone());
        let checksum = world.checksum();
        self.session = Some(Session { world, revision: 0 });
        Ok(json!({"session_id":id,"revision":0,"checksum":checksum}))
    }
    fn session_close(&mut self, params: Value) -> ProtocolResult<Value> {
        let params = parse::<SessionRef>(params)?;
        self.session_ref(&params.session_id)?;
        self.session = None;
        Ok(json!({"closed":true}))
    }
    fn inspect(&self, params: Value) -> ProtocolResult<Value> {
        let params = parse::<SessionRef>(params)?;
        let session = self.session_ref(&params.session_id)?;
        Ok(
            json!({"revision":session.revision,"checksum":session.world.checksum(),"snapshot":snapshot(&session.world)?}),
        )
    }
    fn mutate(&mut self, params: Value) -> ProtocolResult<Value> {
        require(self.policy.capabilities.world_mutate, "world_mutate")?;
        let params = parse::<MutateParams>(params)?;
        self.check_budget(params.ticks, params.events.len())?;
        let session = self.session_mut(&params.session_id)?;
        check_revision(session, params.expected_revision)?;
        let before = session.world.checksum();
        run_ticks(&mut session.world, params.ticks, &params.events).map_err(invalid)?;
        session.revision += 1;
        Ok(
            json!({"revision":session.revision,"checksum_before":before,"checksum_after":session.world.checksum(),"tick":session.world.tick}),
        )
    }
    fn input(&mut self, params: Value) -> ProtocolResult<Value> {
        require(self.policy.capabilities.world_mutate, "world_mutate")?;
        let params = parse::<InputParams>(params)?;
        self.check_budget(0, params.events.len())?;
        let session = self.session_mut(&params.session_id)?;
        check_revision(session, params.expected_revision)?;
        let before = session.world.checksum();
        for event in &params.events {
            session.world.apply_event(event).map_err(invalid)?;
        }
        session.revision += 1;
        Ok(
            json!({"revision":session.revision,"checksum_before":before,"checksum_after":session.world.checksum(),"events":params.events.len()}),
        )
    }
    fn capture(&mut self, params: Value) -> ProtocolResult<Value> {
        require_capture(&self.policy.capabilities)?;
        let params = parse::<CaptureParams>(params)?;
        let target = confined_path(&self.root, Path::new(&params.path))?;
        let render = self.render.clone();
        let root = self.root.clone();
        let session = self.session_mut(&params.session_id)?;
        check_revision(session, params.expected_revision)?;
        let textures = BTreeMap::new();
        let manifest = capture::capture(
            &session.world,
            &render,
            &target,
            params.format,
            &textures,
            &capture::Channels::default(),
        )
        .map_err(invalid)?;
        session.revision += 1;
        Ok(
            json!({"revision":session.revision,"path":relative(&root,&target),"manifest":relative(&root,&manifest),"checksum":session.world.checksum()}),
        )
    }
    fn capture_multi(&mut self, params: Value) -> ProtocolResult<Value> {
        require_capture(&self.policy.capabilities)?;
        let params = parse::<MultiParams>(params)?;
        capture::validate_views(&params.views).map_err(invalid)?;
        if params.views.views.len() > self.policy.limits.max_captures {
            return Err(quota("max_captures"));
        }
        let target = confined_path(&self.root, Path::new(&params.output_dir))?;
        let render = self.render.clone();
        let session = self.session_mut(&params.session_id)?;
        check_revision(session, params.expected_revision)?;
        if params.dry_run {
            return Ok(
                json!({"committed":false,"revision":session.revision,"files":planned_multi(&target,&params.views)}),
            );
        }
        let textures = BTreeMap::new();
        let manifest = capture::capture_multi(
            &session.world,
            &render,
            &params.views,
            &target,
            &textures,
            &capture::Channels::default(),
        )
        .map_err(invalid)?;
        session.revision += 1;
        Ok(
            json!({"committed":true,"revision":session.revision,"manifest":relative(&self.root,&manifest),"files":planned_multi(&target,&params.views)}),
        )
    }
    fn state_diff(&self, params: Value) -> ProtocolResult<Value> {
        let params = parse::<DiffParams>(params)?;
        let session = self.session_ref(&params.session_id)?;
        Ok(diff::compare_values(
            &params.snapshot,
            &snapshot(&session.world)?,
        ))
    }

    fn transaction(&mut self, params: Value) -> ProtocolResult<Value> {
        require(self.policy.capabilities.world_mutate, "world_mutate")?;
        let params = parse::<TransactionParams>(params)?;
        if params.operations.len() > self.policy.limits.max_operations {
            return Err(quota("max_operations"));
        }
        let current = self.session_ref(&params.session_id)?;
        check_revision(current, params.expected_revision)?;
        let mut world = current.world.clone();
        let revision_before = current.revision;
        let checksum_before = world.checksum();
        let mut captures = Vec::new();
        let mut total_ticks = 0u64;
        let mut total_events = 0usize;
        let mut capture_count = 0usize;
        for (index, operation) in params.operations.iter().enumerate() {
            let result = self.apply_operation(
                &mut world,
                operation,
                &mut captures,
                &mut total_ticks,
                &mut total_events,
                &mut capture_count,
            );
            if let Err(error) = result {
                return Err(ProtocolError::new("transaction_aborted", error.message)
                    .details(json!({"operation_index":index,"cause":error.code})));
            }
        }
        let files = planned_files(&captures, &self.root);
        let difference = diff::compare_values(&snapshot(&current.world)?, &snapshot(&world)?);
        if params.dry_run {
            return Ok(
                json!({"committed":false,"revision_before":revision_before,"revision_after":revision_before,"checksum_before":checksum_before,"checksum_after":world.checksum(),"ticks":total_ticks,"events":total_events,"files":files,"diff":difference}),
            );
        }
        let stage = self
            .root
            .join(format!(".aetherion-staging-{}", std::process::id()));
        let publication = self
            .render_staging(&world, &captures, &stage)
            .and_then(|_| self.publish_staging(&captures, &stage));
        if let Err(error) = publication {
            let _ = fs::remove_dir_all(&stage);
            return Err(ProtocolError::new("transaction_aborted", error.message));
        }
        let _ = fs::remove_dir_all(&stage);
        let checksum_after = world.checksum();
        let session = self.session_mut(&params.session_id)?;
        session.world = world;
        session.revision += 1;
        Ok(
            json!({"committed":true,"revision_before":revision_before,"revision_after":session.revision,"checksum_before":checksum_before,"checksum_after":checksum_after,"ticks":total_ticks,"events":total_events,"files":files,"diff":difference}),
        )
    }

    fn apply_operation(
        &self,
        world: &mut World,
        operation: &Operation,
        captures: &mut Vec<PreparedCapture>,
        total_ticks: &mut u64,
        total_events: &mut usize,
        capture_count: &mut usize,
    ) -> ProtocolResult<()> {
        match operation.method.as_str() {
            "world.step" | "world.run" => {
                let value = parse::<StepOperation>(operation.params.clone())?;
                *total_ticks = total_ticks
                    .checked_add(value.ticks)
                    .ok_or_else(|| quota("max_ticks_per_request"))?;
                *total_events = total_events
                    .checked_add(value.events.len())
                    .ok_or_else(|| quota("max_events"))?;
                self.check_budget(*total_ticks, *total_events)?;
                run_ticks(world, value.ticks, &value.events).map_err(invalid)
            }
            "input.apply" => {
                let value = parse::<InputOperation>(operation.params.clone())?;
                *total_events = total_events
                    .checked_add(value.events.len())
                    .ok_or_else(|| quota("max_events"))?;
                self.check_budget(*total_ticks, *total_events)?;
                for event in &value.events {
                    world.apply_event(event).map_err(invalid)?;
                }
                Ok(())
            }
            "capture.create" => {
                require_capture(&self.policy.capabilities)?;
                let value = parse::<CaptureOperation>(operation.params.clone())?;
                *capture_count += 1;
                if *capture_count > self.policy.limits.max_captures {
                    return Err(quota("max_captures"));
                }
                captures.push(PreparedCapture::Single {
                    target: confined_path(&self.root, Path::new(&value.path))?,
                    format: value.format,
                });
                Ok(())
            }
            "capture.multi" => {
                require_capture(&self.policy.capabilities)?;
                let value = parse::<MultiOperation>(operation.params.clone())?;
                capture::validate_views(&value.views).map_err(invalid)?;
                *capture_count = capture_count
                    .checked_add(value.views.views.len())
                    .ok_or_else(|| quota("max_captures"))?;
                if *capture_count > self.policy.limits.max_captures {
                    return Err(quota("max_captures"));
                }
                captures.push(PreparedCapture::Multi {
                    target: confined_path(&self.root, Path::new(&value.output_dir))?,
                    views: value.views,
                });
                Ok(())
            }
            _ => Err(ProtocolError::new(
                "method_not_found",
                format!("opération interdite: {}", operation.method),
            )),
        }
    }

    fn render_staging(
        &self,
        world: &World,
        captures: &[PreparedCapture],
        stage: &Path,
    ) -> ProtocolResult<()> {
        if captures.is_empty() {
            return Ok(());
        }
        if stage.exists() {
            fs::remove_dir_all(stage).map_err(|e| invalid(format!("nettoyage staging: {e}")))?;
        }
        fs::create_dir(stage).map_err(|e| invalid(format!("création staging: {e}")))?;
        let textures = BTreeMap::new();
        for (index, item) in captures.iter().enumerate() {
            match item {
                PreparedCapture::Single { format, .. } => {
                    capture::capture(
                        world,
                        &self.render,
                        &stage.join(format!("single-{index}.{}", format.extension())),
                        *format,
                        &textures,
                        &capture::Channels::default(),
                    )
                    .map_err(invalid)?;
                }
                PreparedCapture::Multi { views, .. } => {
                    capture::capture_multi(
                        world,
                        &self.render,
                        views,
                        &stage.join(format!("multi-{index}")),
                        &textures,
                        &capture::Channels::default(),
                    )
                    .map_err(invalid)?;
                }
            }
        }
        Ok(())
    }
    fn publish_staging(&self, captures: &[PreparedCapture], stage: &Path) -> ProtocolResult<()> {
        for item in captures {
            match item {
                PreparedCapture::Single { target, .. } => {
                    if target.exists() || crate::render::manifest_path(target).exists() {
                        return Err(invalid("la capture cible existe déjà"));
                    }
                }
                PreparedCapture::Multi { target, .. } if target.exists() => {
                    return Err(invalid("le dossier cible existe déjà"));
                }
                _ => {}
            }
        }
        let mut published = Vec::new();
        for (index, item) in captures.iter().enumerate() {
            let result: ProtocolResult<()> = match item {
                PreparedCapture::Single { target, format } => {
                    let source = stage.join(format!("single-{index}.{}", format.extension()));
                    let source_manifest = crate::render::manifest_path(&source);
                    let target_manifest = crate::render::manifest_path(target);
                    ensure_parent(target)?;
                    fs::rename(&source, target)
                        .map_err(|e| invalid(format!("publication capture: {e}")))?;
                    published.push(target.clone());
                    fs::rename(&source_manifest, &target_manifest)
                        .map_err(|e| invalid(format!("publication manifeste: {e}")))?;
                    published.push(target_manifest);
                    Ok(())
                }
                PreparedCapture::Multi { target, .. } => {
                    ensure_parent(target)?;
                    fs::rename(stage.join(format!("multi-{index}")), target)
                        .map_err(|e| invalid(format!("publication multi-vues: {e}")))?;
                    published.push(target.clone());
                    Ok(())
                }
            };
            if let Err(error) = result {
                for path in published.iter().rev() {
                    let _ = if path.is_dir() {
                        fs::remove_dir_all(path)
                    } else {
                        fs::remove_file(path)
                    };
                }
                return Err(error);
            }
        }
        Ok(())
    }

    fn session_ref(&self, id: &str) -> ProtocolResult<&Session> {
        if id != "session-1" {
            return Err(ProtocolError::new("session_not_found", "session inconnue"));
        }
        self.session
            .as_ref()
            .ok_or_else(|| ProtocolError::new("session_not_found", "session inconnue"))
    }
    fn session_mut(&mut self, id: &str) -> ProtocolResult<&mut Session> {
        if id != "session-1" {
            return Err(ProtocolError::new("session_not_found", "session inconnue"));
        }
        self.session
            .as_mut()
            .ok_or_else(|| ProtocolError::new("session_not_found", "session inconnue"))
    }
    fn check_budget(&self, ticks: u64, events: usize) -> ProtocolResult<()> {
        if ticks > self.policy.limits.max_ticks_per_request {
            return Err(quota("max_ticks_per_request"));
        }
        if events > self.policy.limits.max_events {
            return Err(quota("max_events"));
        }
        Ok(())
    }
    fn write_audit(&self, request_id: &str, method: &str, verdict: &str) {
        let Some(path) = &self.audit else {
            return;
        };
        let line = json!({"schema":"aetherion.agent-audit/v1","request_id":request_id,"method":method,"verdict":verdict}).to_string() + "\n";
        let size = fs::metadata(path).map(|v| v.len()).unwrap_or(0);
        if size + line.len() as u64 > self.policy.limits.max_audit_bytes {
            return;
        }
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

fn parse<T: for<'de> Deserialize<'de>>(value: Value) -> ProtocolResult<T> {
    serde_json::from_value(value)
        .map_err(|e| ProtocolError::new("invalid_request", format!("paramètres invalides: {e}")))
}
fn require(enabled: bool, name: &str) -> ProtocolResult<()> {
    if enabled {
        Ok(())
    } else {
        Err(ProtocolError::new(
            "capability_denied",
            format!("capacité refusée: {name}"),
        ))
    }
}
fn require_capture(capabilities: &Capabilities) -> ProtocolResult<()> {
    require(capabilities.capture, "capture")?;
    require(capabilities.file_write, "file_write")
}
fn quota(name: &str) -> ProtocolError {
    ProtocolError::new("quota_exceeded", format!("quota dépassé: {name}"))
}
fn invalid(error: impl ToString) -> ProtocolError {
    ProtocolError::new("invalid_request", error.to_string())
}
fn check_revision(session: &Session, expected: Option<u64>) -> ProtocolResult<()> {
    if expected.is_some_and(|value| value != session.revision) {
        Err(ProtocolError::new("stale_revision", "révision obsolète")
            .details(json!({"expected":expected,"actual":session.revision})))
    } else {
        Ok(())
    }
}
fn snapshot(world: &World) -> ProtocolResult<Value> {
    serde_json::from_str(&world.snapshot_json().map_err(invalid)?).map_err(invalid)
}
fn run_ticks(world: &mut World, ticks: u64, events: &[InputEvent]) -> Result<()> {
    let mut index = 0;
    for _ in 0..ticks {
        let start = index;
        while index < events.len() && events[index].tick == world.tick {
            index += 1;
        }
        world.step_with_events(&events[start..index])?;
    }
    if index != events.len() {
        return Err("événement hors de la plage de ticks".into());
    }
    Ok(())
}
fn error_response(id: &str, error: ProtocolError) -> Value {
    json!({"schema":RESPONSE_SCHEMA,"request_id":id,"status":"error","error":error})
}
fn confined_path(root: &Path, requested: &Path) -> ProtocolResult<PathBuf> {
    if requested.as_os_str().is_empty() {
        return Err(invalid("chemin vide"));
    }
    if requested.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::Prefix(_) | Component::RootDir
        )
    }) {
        return Err(invalid("chemin hors racine ou traversal interdit"));
    }
    let target = root.join(requested);
    let mut ancestor = target.parent();
    while let Some(path) = ancestor {
        if path.exists() {
            let canonical = fs::canonicalize(path).map_err(invalid)?;
            if !canonical.starts_with(root) {
                return Err(invalid("chemin hors racine"));
            }
            break;
        }
        ancestor = path.parent();
    }
    Ok(target)
}
fn ensure_parent(path: &Path) -> ProtocolResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(invalid)?;
    }
    Ok(())
}
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
fn planned_multi(target: &Path, views: &ViewsFile) -> Vec<String> {
    let mut files: Vec<String> = views
        .views
        .iter()
        .map(|view| {
            target
                .join(format!("{}.{}", view.name, view.format.extension()))
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    files.push(
        target
            .join("manifest.json")
            .to_string_lossy()
            .replace('\\', "/"),
    );
    files
}
fn planned_files(captures: &[PreparedCapture], root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for item in captures {
        match item {
            PreparedCapture::Single { target, .. } => {
                files.push(relative(root, target));
                files.push(relative(root, &crate::render::manifest_path(target)));
            }
            PreparedCapture::Multi { target, views } => {
                files.extend(
                    planned_multi(target, views)
                        .into_iter()
                        .map(|path| relative(root, Path::new(&path))),
                );
            }
        }
    }
    files
}

#[allow(dead_code)]
fn _ordered_object() -> BTreeMap<String, Value> {
    BTreeMap::new()
}
