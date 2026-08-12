use std::collections::BTreeSet;
use std::fmt::Display;
use std::fs;
use std::path::Path;

use wasmi::{Caller, Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder};

use crate::assets::AssetManager;
use crate::plugin::{self, Capability, PluginManifest};
use crate::scene::Scene;
use crate::simulation::World;
use crate::{AppError, Result};

pub const DEFAULT_ENTRYPOINT: &str = "aetherion_main";
pub const HOST_MODULE: &str = "aetherion_v1";
pub const HOST_API_VERSION: &str = "aetherion.host/v1";
pub const MAX_TELEMETRY_RECORDS: usize = 1024;
pub const IO_READ_QUOTA_ERROR: &str = "plugin_runtime_io_read_quota";
pub const IO_WRITE_QUOTA_ERROR: &str = "plugin_runtime_io_write_quota";
pub const FILES_QUOTA_ERROR: &str = "plugin_runtime_files_quota";

const SIMULATION_TICK: &str = "simulation_tick";
const SIMULATION_CHECKSUM: &str = "simulation_checksum";
const SIMULATION_ENTITY_COUNT: &str = "simulation_entity_count";
const SIMULATION_ENTITY_FIELD: &str = "simulation_entity_field";
const SCENE_ENTITY_COUNT: &str = "scene_entity_count";
const SCENE_ASSET_COUNT: &str = "scene_asset_count";
const ASSET_COUNT: &str = "asset_count";
const ASSET_SIZE: &str = "asset_size";
const ASSET_READ_BYTE: &str = "asset_read_byte";
const TELEMETRY_WRITE: &str = "telemetry_write";
const TELEMETRY_LEN: &str = "telemetry_len";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeLimits {
    pub fuel: u64,
    pub memory_bytes: u64,
}

impl RuntimeLimits {
    pub const fn unbounded() -> Self {
        Self {
            fuel: u64::MAX,
            memory_bytes: usize::MAX as u64,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeIoLimits {
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub files: u32,
}

impl RuntimeIoLimits {
    pub const fn unbounded() -> Self {
        Self {
            read_bytes: u64::MAX,
            write_bytes: u64::MAX,
            files: u32::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IoUsage {
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub files: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulationEntityView {
    pub id: u64,
    pub position_x: i64,
    pub position_y: i64,
    pub velocity_x: i64,
    pub velocity_y: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulationView {
    pub tick: u64,
    pub checksum: u64,
    pub entities: Vec<SimulationEntityView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneView {
    pub entity_count: usize,
    pub asset_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TelemetryRecord {
    pub key: i64,
    pub value: i64,
}

/// Données explicitement sélectionnées par l'hôte pour une exécution.
///
/// Le runtime conserve des copies immuables des vues de simulation, de scène et
/// d'assets. Un plugin ne reçoit donc ni référence Rust, ni chemin de fichier,
/// ni accès mutable au monde.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HostContext {
    pub simulation: Option<SimulationView>,
    pub scene: Option<SceneView>,
    pub assets: Vec<HostAsset>,
    telemetry: Vec<TelemetryRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostAsset {
    pub id: String,
    pub bytes: Vec<u8>,
}

impl HostContext {
    pub fn from_world(world: &World) -> Self {
        let entities = world
            .entities()
            .map(|entity| SimulationEntityView {
                id: entity.id,
                position_x: entity.position.x,
                position_y: entity.position.y,
                velocity_x: entity.velocity.x,
                velocity_y: entity.velocity.y,
            })
            .collect();
        Self {
            simulation: Some(SimulationView {
                tick: world.tick,
                checksum: world.checksum(),
                entities,
            }),
            ..Self::default()
        }
    }

    pub fn with_scene(mut self, scene: &Scene) -> Result<Self> {
        crate::scene::validate(scene)?;
        let mut asset_ids = scene.assets.clone();
        asset_ids.sort();
        self.scene = Some(SceneView {
            entity_count: scene.entities.len(),
            asset_ids,
        });
        Ok(self)
    }

    pub fn with_asset_bytes(mut self, id: impl Into<String>, bytes: Vec<u8>) -> Result<Self> {
        let id = id.into();
        validate_host_asset_id(&id)?;
        if self.assets.iter().any(|asset| asset.id == id) {
            return Err(format!("plugin_runtime_asset_duplicate: {id}").into());
        }
        self.assets.push(HostAsset { id, bytes });
        self.assets.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(self)
    }

    /// Copie uniquement les assets demandés après les validations du manifeste
    /// d'assets : chemin relatif, confinement canonique, taille et checksum.
    pub fn with_assets_from_manager(
        mut self,
        manager: &AssetManager,
        ids: impl IntoIterator<Item = String>,
    ) -> Result<Self> {
        let mut ids: Vec<String> = ids.into_iter().collect();
        ids.sort();
        ids.dedup();
        for id in ids {
            self = self.with_asset_bytes(id.clone(), manager.read_bytes(&id)?)?;
        }
        Ok(self)
    }

    pub fn telemetry(&self) -> &[TelemetryRecord] {
        &self.telemetry
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionResult {
    pub return_code: i32,
    pub fuel_consumed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionReport {
    pub result: ExecutionResult,
    pub telemetry: Vec<TelemetryRecord>,
    pub io: IoUsage,
}

struct IoAccounting {
    limits: RuntimeIoLimits,
    usage: IoUsage,
}

impl IoAccounting {
    fn new(limits: RuntimeIoLimits, selected_files: usize) -> Result<Self> {
        let files = u32::try_from(selected_files).map_err(|_| FILES_QUOTA_ERROR)?;
        if files > limits.files {
            return Err(format!(
                "{FILES_QUOTA_ERROR}: {} fichiers sélectionnés, plafond {}",
                files, limits.files
            )
            .into());
        }
        Ok(Self {
            limits,
            usage: IoUsage {
                files,
                ..IoUsage::default()
            },
        })
    }

    fn consume_read(&mut self, bytes: u64) -> Result<()> {
        let next = self
            .usage
            .read_bytes
            .checked_add(bytes)
            .ok_or(IO_READ_QUOTA_ERROR)?;
        if next > self.limits.read_bytes {
            return Err(IO_READ_QUOTA_ERROR.into());
        }
        self.usage.read_bytes = next;
        Ok(())
    }
}

struct RuntimeState {
    limits: StoreLimits,
    host: HostContext,
    io: IoAccounting,
}

pub fn execute_file(path: &Path, export: &str) -> Result<ExecutionResult> {
    execute_file_with_limits(path, export, RuntimeLimits::unbounded())
}

pub fn execute_file_with_limits(
    path: &Path,
    export: &str,
    limits: RuntimeLimits,
) -> Result<ExecutionResult> {
    let bytes = fs::read(path)
        .map_err(|error| format!("plugin_runtime_read: {}: {error}", path.display()))?;
    execute_bytes_with_limits(&bytes, export, limits)
}

pub fn execute_bytes(bytes: &[u8], export: &str) -> Result<ExecutionResult> {
    execute_bytes_with_limits(bytes, export, RuntimeLimits::unbounded())
}

pub fn execute_bytes_with_limits(
    bytes: &[u8],
    export: &str,
    limits: RuntimeLimits,
) -> Result<ExecutionResult> {
    execute_bytes_with_context(bytes, export, limits, HostContext::default())
}

pub fn execute_bytes_with_context(
    bytes: &[u8],
    export: &str,
    limits: RuntimeLimits,
    host: HostContext,
) -> Result<ExecutionResult> {
    execute_internal(
        bytes,
        export,
        limits,
        RuntimeIoLimits::unbounded(),
        BTreeSet::new(),
        host,
    )
    .map(|report| report.result)
}

pub fn execute_bytes_with_manifest(
    bytes: &[u8],
    export: &str,
    manifest: &PluginManifest,
    host: HostContext,
) -> Result<ExecutionReport> {
    plugin::validate(manifest)?;
    let limits = RuntimeLimits {
        fuel: manifest.quotas.fuel,
        memory_bytes: manifest.quotas.memory_bytes,
    };
    let io_limits = RuntimeIoLimits {
        read_bytes: manifest.quotas.io_read_bytes,
        write_bytes: manifest.quotas.io_write_bytes,
        files: manifest.quotas.files,
    };
    let capabilities = manifest.capabilities.iter().copied().collect();
    execute_internal(bytes, export, limits, io_limits, capabilities, host)
}

pub fn execute_file_with_manifest(
    path: &Path,
    export: &str,
    manifest: &PluginManifest,
    host: HostContext,
) -> Result<ExecutionReport> {
    let bytes = fs::read(path)
        .map_err(|error| format!("plugin_runtime_read: {}: {error}", path.display()))?;
    execute_bytes_with_manifest(&bytes, export, manifest, host)
}

fn execute_internal(
    bytes: &[u8],
    export: &str,
    limits: RuntimeLimits,
    io_limits: RuntimeIoLimits,
    capabilities: BTreeSet<Capability>,
    host: HostContext,
) -> Result<ExecutionReport> {
    if export.is_empty() || export.len() > 128 {
        return Err("plugin_runtime_export_invalid".into());
    }
    if bytes.is_empty() {
        return Err("plugin_runtime_module_empty".into());
    }
    let memory_bytes =
        usize::try_from(limits.memory_bytes).map_err(|_| "plugin_runtime_memory_limit_invalid")?;

    let mut config = Config::default();
    config.consume_fuel(true);
    let engine = Engine::new(&config);
    let module = Module::new(&engine, bytes)
        .map_err(|error| format_error("plugin_runtime_compile", error))?;
    validate_imports(&module, &capabilities)?;

    let store_limits = StoreLimitsBuilder::new()
        .memory_size(memory_bytes)
        .instances(1)
        .memories(1)
        .tables(1)
        .trap_on_grow_failure(true)
        .build();
    let io = IoAccounting::new(io_limits, host.assets.len())?;
    let mut store = Store::new(
        &engine,
        RuntimeState {
            limits: store_limits,
            host,
            io,
        },
    );
    store.limiter(|state| &mut state.limits);
    store
        .set_fuel(limits.fuel)
        .map_err(|error| format_error("plugin_runtime_fuel_config", error))?;

    let mut linker = Linker::<RuntimeState>::new(&engine);
    register_imports(&mut linker, &capabilities)?;
    let instance = linker
        .instantiate(&mut store, &module)
        .map_err(classify_runtime_error("plugin_runtime_instantiate"))?
        .start(&mut store)
        .map_err(classify_runtime_error("plugin_runtime_start"))?;
    let entrypoint = instance
        .get_typed_func::<(), i32>(&store, export)
        .map_err(|error| format_error("plugin_runtime_export", error))?;
    let return_code = entrypoint
        .call(&mut store, ())
        .map_err(classify_runtime_error("plugin_runtime_trap"))?;
    let remaining = store
        .get_fuel()
        .map_err(|error| format_error("plugin_runtime_fuel_read", error))?;
    let fuel_consumed = limits
        .fuel
        .checked_sub(remaining)
        .ok_or("plugin_runtime_fuel_accounting")?;
    let result = ExecutionResult {
        return_code,
        fuel_consumed,
    };
    Ok(ExecutionReport {
        result,
        telemetry: store.data().host.telemetry.clone(),
        io: store.data().io.usage,
    })
}

fn validate_imports(module: &Module, capabilities: &BTreeSet<Capability>) -> Result<()> {
    for import in module.imports() {
        if import.module() != HOST_MODULE {
            return Err(format!(
                "plugin_runtime_import_denied: {}/{}",
                import.module(),
                import.name()
            )
            .into());
        }
        let Some(capability) = capability_for_import(import.name()) else {
            return Err(format!("plugin_runtime_import_unknown: {}", import.name()).into());
        };
        if !capabilities.contains(&capability) {
            return Err(format!(
                "plugin_runtime_capability_denied: {} requires {:?}",
                import.name(),
                capability
            )
            .into());
        }
    }
    Ok(())
}

fn register_imports(
    linker: &mut Linker<RuntimeState>,
    capabilities: &BTreeSet<Capability>,
) -> Result<()> {
    if capabilities.contains(&Capability::SimulationRead) {
        linker
            .func_wrap(
                HOST_MODULE,
                SIMULATION_TICK,
                |caller: Caller<'_, RuntimeState>| {
                    caller
                        .data()
                        .host
                        .simulation
                        .as_ref()
                        .map_or(-1, |view| view.tick as i64)
                },
            )
            .map_err(|error| format_error("plugin_runtime_import_register", error))?;
        linker
            .func_wrap(
                HOST_MODULE,
                SIMULATION_CHECKSUM,
                |caller: Caller<'_, RuntimeState>| {
                    caller
                        .data()
                        .host
                        .simulation
                        .as_ref()
                        .map_or(-1, |view| view.checksum as i64)
                },
            )
            .map_err(|error| format_error("plugin_runtime_import_register", error))?;
        linker
            .func_wrap(
                HOST_MODULE,
                SIMULATION_ENTITY_COUNT,
                |caller: Caller<'_, RuntimeState>| {
                    caller
                        .data()
                        .host
                        .simulation
                        .as_ref()
                        .map_or(-1, |view| i32::try_from(view.entities.len()).unwrap_or(-1))
                },
            )
            .map_err(|error| format_error("plugin_runtime_import_register", error))?;
        linker
            .func_wrap(
                HOST_MODULE,
                SIMULATION_ENTITY_FIELD,
                |caller: Caller<'_, RuntimeState>, index: i32, field: i32| {
                    let Some(view) = caller.data().host.simulation.as_ref() else {
                        return -1;
                    };
                    let Some(entity) = index
                        .try_into()
                        .ok()
                        .and_then(|index: usize| view.entities.get(index))
                    else {
                        return -1;
                    };
                    match field {
                        0 => entity.id as i64,
                        1 => entity.position_x,
                        2 => entity.position_y,
                        3 => entity.velocity_x,
                        4 => entity.velocity_y,
                        _ => -1,
                    }
                },
            )
            .map_err(|error| format_error("plugin_runtime_import_register", error))?;
    }

    if capabilities.contains(&Capability::SceneRead) {
        linker
            .func_wrap(
                HOST_MODULE,
                SCENE_ENTITY_COUNT,
                |caller: Caller<'_, RuntimeState>| {
                    caller
                        .data()
                        .host
                        .scene
                        .as_ref()
                        .map_or(-1, |view| i32::try_from(view.entity_count).unwrap_or(-1))
                },
            )
            .map_err(|error| format_error("plugin_runtime_import_register", error))?;
        linker
            .func_wrap(
                HOST_MODULE,
                SCENE_ASSET_COUNT,
                |caller: Caller<'_, RuntimeState>| {
                    caller
                        .data()
                        .host
                        .scene
                        .as_ref()
                        .map_or(-1, |view| i32::try_from(view.asset_ids.len()).unwrap_or(-1))
                },
            )
            .map_err(|error| format_error("plugin_runtime_import_register", error))?;
    }

    if capabilities.contains(&Capability::AssetRead) {
        linker
            .func_wrap(
                HOST_MODULE,
                ASSET_COUNT,
                |caller: Caller<'_, RuntimeState>| {
                    i32::try_from(caller.data().host.assets.len()).unwrap_or(-1)
                },
            )
            .map_err(|error| format_error("plugin_runtime_import_register", error))?;
        linker
            .func_wrap(
                HOST_MODULE,
                ASSET_SIZE,
                |caller: Caller<'_, RuntimeState>, index: i32| {
                    let Some(asset) = index
                        .try_into()
                        .ok()
                        .and_then(|index: usize| caller.data().host.assets.get(index))
                    else {
                        return -1;
                    };
                    i64::try_from(asset.bytes.len()).unwrap_or(-1)
                },
            )
            .map_err(|error| format_error("plugin_runtime_import_register", error))?;
        linker
            .func_wrap(
                HOST_MODULE,
                ASSET_READ_BYTE,
                |mut caller: Caller<'_, RuntimeState>, index: i32, offset: i32| {
                    let byte = {
                        let Some(asset) = index
                            .try_into()
                            .ok()
                            .and_then(|index: usize| caller.data().host.assets.get(index))
                        else {
                            return Ok(-1);
                        };
                        let Some(byte) = offset
                            .try_into()
                            .ok()
                            .and_then(|offset: usize| asset.bytes.get(offset))
                        else {
                            return Ok(-1);
                        };
                        *byte
                    };
                    caller
                        .data_mut()
                        .io
                        .consume_read(1)
                        .map_err(|error| wasmi::Error::new(error.to_string()))?;
                    Ok(i32::from(byte))
                },
            )
            .map_err(|error| format_error("plugin_runtime_import_register", error))?;
    }

    if capabilities.contains(&Capability::TelemetryWrite) {
        linker
            .func_wrap(
                HOST_MODULE,
                TELEMETRY_WRITE,
                |mut caller: Caller<'_, RuntimeState>, key: i64, value: i64| {
                    let host = &mut caller.data_mut().host;
                    if host.telemetry.len() >= MAX_TELEMETRY_RECORDS {
                        return -1;
                    }
                    host.telemetry.push(TelemetryRecord { key, value });
                    0
                },
            )
            .map_err(|error| format_error("plugin_runtime_import_register", error))?;
        linker
            .func_wrap(
                HOST_MODULE,
                TELEMETRY_LEN,
                |caller: Caller<'_, RuntimeState>| {
                    i32::try_from(caller.data().host.telemetry.len()).unwrap_or(-1)
                },
            )
            .map_err(|error| format_error("plugin_runtime_import_register", error))?;
    }
    Ok(())
}

fn capability_for_import(name: &str) -> Option<Capability> {
    match name {
        SIMULATION_TICK
        | SIMULATION_CHECKSUM
        | SIMULATION_ENTITY_COUNT
        | SIMULATION_ENTITY_FIELD => Some(Capability::SimulationRead),
        SCENE_ENTITY_COUNT | SCENE_ASSET_COUNT => Some(Capability::SceneRead),
        ASSET_COUNT | ASSET_SIZE | ASSET_READ_BYTE => Some(Capability::AssetRead),
        TELEMETRY_WRITE | TELEMETRY_LEN => Some(Capability::TelemetryWrite),
        _ => None,
    }
}

fn validate_host_asset_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(format!("plugin_runtime_asset_id_invalid: {id}").into());
    }
    Ok(())
}

fn format_error(prefix: &str, error: impl Display) -> AppError {
    format!("{prefix}: {error}").into()
}

fn classify_runtime_error(prefix: &'static str) -> impl FnOnce(wasmi::Error) -> AppError {
    move |error| {
        let text = error.to_string();
        let lower = text.to_ascii_lowercase();
        if lower.contains("fuel") {
            format!("plugin_runtime_fuel_exhausted: {text}").into()
        } else if lower.starts_with(IO_READ_QUOTA_ERROR)
            || lower.starts_with(IO_WRITE_QUOTA_ERROR)
            || lower.starts_with(FILES_QUOTA_ERROR)
        {
            text.into()
        } else if lower.contains("memory") || lower.contains("growth operation limited") {
            format!("plugin_runtime_memory_limit: {text}").into()
        } else {
            format!("{prefix}: {text}").into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{HOST_ABI_MAJOR, HOST_ABI_MINOR, MANIFEST_SCHEMA, PluginAbi, PluginQuotas};

    // (module (func (export "aetherion_main") (result i32) i32.const 7))
    const RETURN_SEVEN: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
        0x03, 0x02, 0x01, 0x00, 0x07, 0x12, 0x01, 0x0e, 0x61, 0x65, 0x74, 0x68, 0x65, 0x72, 0x69,
        0x6f, 0x6e, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x41,
        0x07, 0x0b,
    ];

    // (module (func (export "aetherion_main") (result i32) (loop (br 0)) unreachable))
    const INFINITE_LOOP: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
        0x03, 0x02, 0x01, 0x00, 0x07, 0x12, 0x01, 0x0e, 0x61, 0x65, 0x74, 0x68, 0x65, 0x72, 0x69,
        0x6f, 0x6e, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x0a, 0x0a, 0x01, 0x08, 0x00, 0x03,
        0x40, 0x0c, 0x00, 0x0b, 0x00, 0x0b,
    ];

    // (module (memory 0) (func (export "aetherion_main") (result i32)
    //   (drop (memory.grow (i32.const 1))) (i32.const 7)))
    const MEMORY_GROW: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
        0x03, 0x02, 0x01, 0x00, 0x05, 0x03, 0x01, 0x00, 0x00, 0x07, 0x12, 0x01, 0x0e, 0x61, 0x65,
        0x74, 0x68, 0x65, 0x72, 0x69, 0x6f, 0x6e, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x0a,
        0x0b, 0x01, 0x09, 0x00, 0x41, 0x01, 0x40, 0x00, 0x1a, 0x41, 0x07, 0x0b,
    ];

    fn limited(fuel: u64, memory_bytes: u64) -> RuntimeLimits {
        RuntimeLimits { fuel, memory_bytes }
    }

    fn manifest(capabilities: Vec<Capability>) -> PluginManifest {
        manifest_with_io(capabilities, 1024, 1024, 1)
    }

    fn manifest_with_io(
        capabilities: Vec<Capability>,
        io_read_bytes: u64,
        io_write_bytes: u64,
        files: u32,
    ) -> PluginManifest {
        PluginManifest {
            schema: MANIFEST_SCHEMA.into(),
            id: "org.aetherion.runtime-test".into(),
            version: "1.0.0".into(),
            abi: PluginAbi {
                major: HOST_ABI_MAJOR,
                minimum_host_minor: HOST_ABI_MINOR,
            },
            capabilities,
            quotas: PluginQuotas {
                memory_bytes: 65536,
                fuel: 1000,
                io_read_bytes,
                io_write_bytes,
                files,
            },
        }
    }

    fn leb(mut value: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                return bytes;
            }
        }
    }

    fn wasm_with_import(import_name: &str, function_type: &[u8], body: &[u8]) -> Vec<u8> {
        wasm_with_import_from(HOST_MODULE, import_name, function_type, body)
    }

    fn wasm_with_import_from(
        module_name: &str,
        import_name: &str,
        function_type: &[u8],
        body: &[u8],
    ) -> Vec<u8> {
        fn string(bytes: &mut Vec<u8>, value: &str) {
            bytes.extend(leb(value.len() as u32));
            bytes.extend(value.as_bytes());
        }
        fn section(bytes: &mut Vec<u8>, id: u8, payload: &[u8]) {
            bytes.push(id);
            bytes.extend(leb(payload.len() as u32));
            bytes.extend(payload);
        }

        let mut module = b"\0asm\x01\0\0\0".to_vec();
        let mut types = vec![2];
        types.extend(function_type);
        types.extend([0x60, 0, 1, 0x7f]);
        section(&mut module, 1, &types);

        let mut imports = vec![1];
        string(&mut imports, module_name);
        string(&mut imports, import_name);
        imports.extend([0, 0]);
        section(&mut module, 2, &imports);

        section(&mut module, 3, &[1, 1]);
        let mut exports = vec![1];
        string(&mut exports, DEFAULT_ENTRYPOINT);
        exports.extend([0, 1]);
        section(&mut module, 7, &exports);

        let mut code = vec![1];
        let mut function = vec![0];
        function.extend(body);
        function.push(0x0b);
        code.extend(leb(function.len() as u32));
        code.extend(function);
        section(&mut module, 10, &code);
        module
    }

    #[test]
    fn executes_a_module_without_host_imports_and_reports_fuel() {
        let first =
            execute_bytes_with_limits(RETURN_SEVEN, DEFAULT_ENTRYPOINT, limited(1000, 65536))
                .unwrap();
        let second =
            execute_bytes_with_limits(RETURN_SEVEN, DEFAULT_ENTRYPOINT, limited(1000, 65536))
                .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.return_code, 7);
        assert!(first.fuel_consumed > 0);
    }

    #[test]
    fn fuel_exhaustion_is_reported_before_unbounded_execution() {
        let error = execute_bytes_with_limits(INFINITE_LOOP, DEFAULT_ENTRYPOINT, limited(1, 65536))
            .unwrap_err();
        assert!(
            error.message.starts_with("plugin_runtime_fuel_exhausted"),
            "{}",
            error.message
        );
    }

    #[test]
    fn memory_growth_beyond_the_limit_is_reported() {
        let error = execute_bytes_with_limits(MEMORY_GROW, DEFAULT_ENTRYPOINT, limited(1000, 0))
            .unwrap_err();
        assert!(
            error.message.starts_with("plugin_runtime_memory_limit"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rejects_invalid_modules_and_exports_with_stable_prefixes() {
        assert!(
            execute_bytes(&[], DEFAULT_ENTRYPOINT)
                .unwrap_err()
                .message
                .starts_with("plugin_runtime_module_empty")
        );
        assert!(
            execute_bytes(&[0, 1, 2], DEFAULT_ENTRYPOINT)
                .unwrap_err()
                .message
                .starts_with("plugin_runtime_compile")
        );
        assert!(
            execute_bytes(RETURN_SEVEN, "missing")
                .unwrap_err()
                .message
                .starts_with("plugin_runtime_export")
        );
    }

    #[test]
    fn simulation_import_requires_capability_and_reads_a_snapshot() {
        let module = wasm_with_import(SIMULATION_ENTITY_COUNT, &[0x60, 0, 1, 0x7f], &[0x10, 0x00]);
        let project: crate::project::Project =
            toml::from_str(crate::project::Project::example()).unwrap();
        let world = World::from_project(project);
        let checksum_before = world.checksum();
        let denied = execute_bytes_with_manifest(
            &module,
            DEFAULT_ENTRYPOINT,
            &manifest(vec![]),
            HostContext::from_world(&world),
        )
        .unwrap_err();
        assert!(
            denied
                .message
                .starts_with("plugin_runtime_capability_denied")
        );

        let report = execute_bytes_with_manifest(
            &module,
            DEFAULT_ENTRYPOINT,
            &manifest(vec![Capability::SimulationRead]),
            HostContext::from_world(&world),
        )
        .unwrap();
        assert_eq!(report.result.return_code, 2);
        assert_eq!(world.checksum(), checksum_before);
        let repeat = execute_bytes_with_manifest(
            &module,
            DEFAULT_ENTRYPOINT,
            &manifest(vec![Capability::SimulationRead]),
            HostContext::from_world(&world),
        )
        .unwrap();
        assert_eq!(report, repeat);
    }

    #[test]
    fn scene_import_reads_only_the_selected_scene_view() {
        let scene = Scene {
            schema: "aetherion.scene/v1".into(),
            id: "plugin-scene".into(),
            metadata: crate::scene::SceneMetadata::default(),
            camera: crate::project::CameraConfig::default(),
            assets: vec!["texture".into()],
            entities: vec![],
        };
        let module = wasm_with_import(SCENE_ASSET_COUNT, &[0x60, 0, 1, 0x7f], &[0x10, 0x00]);
        let report = execute_bytes_with_manifest(
            &module,
            DEFAULT_ENTRYPOINT,
            &manifest(vec![Capability::SceneRead]),
            HostContext::default().with_scene(&scene).unwrap(),
        )
        .unwrap();
        assert_eq!(report.result.return_code, 1);
    }

    #[test]
    fn asset_import_is_indexed_and_never_receives_a_path() {
        let module = wasm_with_import(ASSET_COUNT, &[0x60, 0, 1, 0x7f], &[0x10, 0x00]);
        let host = HostContext::default()
            .with_asset_bytes("texture", vec![1, 2, 3])
            .unwrap();
        let report = execute_bytes_with_manifest(
            &module,
            DEFAULT_ENTRYPOINT,
            &manifest(vec![Capability::AssetRead]),
            host,
        )
        .unwrap();
        assert_eq!(report.result.return_code, 1);
        let error = HostContext::default()
            .with_asset_bytes("../outside", vec![1])
            .unwrap_err();
        assert!(error.message.starts_with("plugin_runtime_asset_id_invalid"));
    }

    #[test]
    fn io_and_file_quotas_are_enforced_and_reported_deterministically() {
        let module = wasm_with_import(
            ASSET_READ_BYTE,
            &[0x60, 2, 0x7f, 0x7f, 1, 0x7f],
            &[0x41, 0, 0x41, 0, 0x10, 0x00],
        );
        let host = HostContext::default()
            .with_asset_bytes("texture", vec![7])
            .unwrap();
        let allowed = execute_bytes_with_manifest(
            &module,
            DEFAULT_ENTRYPOINT,
            &manifest_with_io(vec![Capability::AssetRead], 1, 0, 1),
            host.clone(),
        )
        .unwrap();
        assert_eq!(allowed.result.return_code, 7);
        assert_eq!(
            allowed.io,
            IoUsage {
                read_bytes: 1,
                write_bytes: 0,
                files: 1
            }
        );

        let denied_read = execute_bytes_with_manifest(
            &module,
            DEFAULT_ENTRYPOINT,
            &manifest_with_io(vec![Capability::AssetRead], 0, 0, 1),
            host.clone(),
        )
        .unwrap_err();
        assert!(denied_read.message.starts_with(IO_READ_QUOTA_ERROR));

        let denied_files = execute_bytes_with_manifest(
            &module,
            DEFAULT_ENTRYPOINT,
            &manifest_with_io(vec![Capability::AssetRead], 1, 0, 0),
            host,
        )
        .unwrap_err();
        assert!(denied_files.message.starts_with(FILES_QUOTA_ERROR));
    }

    #[test]
    fn telemetry_import_only_writes_to_the_memory_report() {
        let module = wasm_with_import(
            TELEMETRY_WRITE,
            &[0x60, 2, 0x7e, 0x7e, 1, 0x7f],
            &[0x42, 7, 0x42, 9, 0x10, 0x00],
        );
        let report = execute_bytes_with_manifest(
            &module,
            DEFAULT_ENTRYPOINT,
            &manifest(vec![Capability::TelemetryWrite]),
            HostContext::default(),
        )
        .unwrap();
        assert_eq!(report.result.return_code, 0);
        assert_eq!(report.telemetry, [TelemetryRecord { key: 7, value: 9 }]);
    }

    #[test]
    fn non_host_imports_are_rejected_even_when_a_capability_exists() {
        let module = wasm_with_import_from(
            "env",
            SIMULATION_ENTITY_COUNT,
            &[0x60, 0, 1, 0x7f],
            &[0x10, 0x00],
        );
        let error = execute_bytes_with_manifest(
            &module,
            DEFAULT_ENTRYPOINT,
            &manifest(vec![Capability::SimulationRead]),
            HostContext::default(),
        )
        .unwrap_err();
        assert!(error.message.starts_with("plugin_runtime_import_denied"));
    }
}
