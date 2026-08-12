use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::Result;
use crate::assets::AssetManager;
use crate::plugin;
use crate::plugin_runtime::{
    self, DEFAULT_ENTRYPOINT, ExecutionReport, HostContext, IoUsage, TelemetryRecord,
};
use crate::project::Project;
use crate::scene;
use crate::simulation::World;

pub const REPORT_SCHEMA: &str = "aetherion.plugin-run-report/v1";

#[derive(Clone, Debug)]
pub struct RunOptions {
    pub manifest: PathBuf,
    pub module: PathBuf,
    pub path: Option<PathBuf>,
    pub scene: Option<String>,
    pub assets: Option<PathBuf>,
    pub export: String,
    pub dry_run: bool,
    pub report: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PluginRunReport {
    pub schema: &'static str,
    pub status: &'static str,
    pub plugin_id: String,
    pub plugin_version: String,
    pub export: String,
    pub dry_run: bool,
    pub return_code: Option<i32>,
    pub fuel_consumed: Option<u64>,
    pub io: Option<IoUsage>,
    pub telemetry: Vec<TelemetryRecord>,
}

pub fn run(options: RunOptions) -> Result<PluginRunReport> {
    let manifest = plugin::load(&options.manifest)?;
    validate_module_file(&options.module)?;
    let host = build_host_context(&options)?;
    if host.assets.len() > manifest.quotas.files as usize {
        return Err(format!(
            "plugin_runtime_files_quota: {} fichiers sélectionnés, plafond {}",
            host.assets.len(),
            manifest.quotas.files
        )
        .into());
    }

    let export = normalized_export(&options.export)?;
    let report = if options.dry_run {
        let bytes = fs::read(&options.module).map_err(|error| {
            format!(
                "plugin_run_module_read: {}: {error}",
                options.module.display()
            )
        })?;
        plugin_runtime::validate_bytes_with_manifest(&bytes, &export, &manifest, &host)?;
        PluginRunReport {
            schema: REPORT_SCHEMA,
            status: "planned",
            plugin_id: manifest.id,
            plugin_version: manifest.version,
            export,
            dry_run: true,
            return_code: None,
            fuel_consumed: None,
            io: None,
            telemetry: Vec::new(),
        }
    } else {
        let execution =
            plugin_runtime::execute_file_with_manifest(&options.module, &export, &manifest, host)?;
        report_from_execution(manifest.id, manifest.version, export, execution)
    };

    if let Some(path) = options.report.as_deref() {
        write_atomic_report(path, &report)?;
    }
    Ok(report)
}

fn report_from_execution(
    plugin_id: String,
    plugin_version: String,
    export: String,
    execution: ExecutionReport,
) -> PluginRunReport {
    PluginRunReport {
        schema: REPORT_SCHEMA,
        status: "executed",
        plugin_id,
        plugin_version,
        export,
        dry_run: false,
        return_code: Some(execution.result.return_code),
        fuel_consumed: Some(execution.result.fuel_consumed),
        io: Some(execution.io),
        telemetry: execution.telemetry,
    }
}

fn normalized_export(export: &str) -> Result<String> {
    let export = if export.is_empty() {
        DEFAULT_ENTRYPOINT
    } else {
        export
    };
    if export.len() > 128 || export.bytes().any(|byte| byte == 0) {
        return Err("plugin_runtime_export_invalid".into());
    }
    Ok(export.to_owned())
}

fn validate_module_file(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("plugin_run_module_read: {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("plugin_run_module_not_regular_file".into());
    }
    if metadata.len() == 0 {
        return Err("plugin_runtime_module_empty".into());
    }
    Ok(())
}

fn build_host_context(options: &RunOptions) -> Result<HostContext> {
    let mut host = HostContext::default();
    if let Some(root) = options.path.as_deref() {
        host = HostContext::from_world(&World::from_project(Project::load(root)?));
    }
    if let Some(scene_id) = options.scene.as_deref() {
        let root = options
            .path
            .as_deref()
            .ok_or("plugin_run_scene_requires_path")?;
        host = host.with_scene(&scene::load(root, scene_id)?)?;
    }
    if let Some(asset_manifest) = options.assets.as_deref() {
        let project_root = options.path.as_deref().unwrap_or_else(|| Path::new("."));
        let (root, requested) = asset_manifest_location(project_root, asset_manifest)?;
        let manager = AssetManager::load(&root, Some(&requested))?;
        let ids = manager.asset_ids().map(str::to_owned).collect::<Vec<_>>();
        host = host.with_assets_from_manager(&manager, ids)?;
    }
    Ok(host)
}

fn asset_manifest_location(project_root: &Path, manifest: &Path) -> Result<(PathBuf, PathBuf)> {
    if manifest.is_absolute() {
        let parent = manifest
            .parent()
            .ok_or("asset_manifest_invalid: parent absent")?;
        let file = manifest
            .file_name()
            .ok_or("asset_manifest_invalid: nom absent")?;
        Ok((parent.to_path_buf(), PathBuf::from(file)))
    } else {
        Ok((project_root.to_path_buf(), manifest.to_path_buf()))
    }
}

fn write_atomic_report(path: &Path, report: &PluginRunReport) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| format!("plugin_run_report_serialize: {error}"))?;
    bytes.push(b'\n');
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("plugin_run_report_mkdir: {}: {error}", parent.display()))?;
    let temporary = parent.join(format!(".aetherion-plugin-run-{}.tmp", std::process::id()));
    fs::write(&temporary, bytes)
        .map_err(|error| format!("plugin_run_report_write: {}: {error}", temporary.display()))?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("plugin_run_report_replace: {}: {error}", path.display()))?;
    }
    fs::rename(&temporary, path)
        .map_err(|error| format!("plugin_run_report_rename: {}: {error}", path.display()).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{
        HOST_ABI_MAJOR, HOST_ABI_MINOR, MANIFEST_SCHEMA, PluginAbi, PluginManifest, PluginQuotas,
    };

    const RETURN_SEVEN: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
        0x03, 0x02, 0x01, 0x00, 0x07, 0x12, 0x01, 0x0e, 0x61, 0x65, 0x74, 0x68, 0x65, 0x72, 0x69,
        0x6f, 0x6e, 0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x41,
        0x07, 0x0b,
    ];

    fn manifest() -> PluginManifest {
        PluginManifest {
            schema: MANIFEST_SCHEMA.into(),
            id: "org.aetherion.cli-test".into(),
            version: "1.0.0".into(),
            abi: PluginAbi {
                major: HOST_ABI_MAJOR,
                minimum_host_minor: HOST_ABI_MINOR,
            },
            capabilities: vec![],
            quotas: PluginQuotas {
                memory_bytes: 65536,
                fuel: 1000,
                io_read_bytes: 0,
                io_write_bytes: 0,
                files: 0,
            },
        }
    }

    fn directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "aetherion-plugin-run-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn dry_run_writes_a_deterministic_atomic_report_without_execution() {
        let root = directory("dry-run");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("plugin.json");
        let module_path = root.join("plugin.wasm");
        let report_path = root.join("report.json");
        fs::write(&manifest_path, serde_json::to_vec(&manifest()).unwrap()).unwrap();
        fs::write(&module_path, RETURN_SEVEN).unwrap();
        let report = run(RunOptions {
            manifest: manifest_path,
            module: module_path,
            path: None,
            scene: None,
            assets: None,
            export: DEFAULT_ENTRYPOINT.into(),
            dry_run: true,
            report: Some(report_path.clone()),
        })
        .unwrap();
        assert_eq!(report.status, "planned");
        assert_eq!(report.return_code, None);
        let bytes = fs::read(report_path).unwrap();
        assert!(bytes.ends_with(b"\n"));
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["schema"], REPORT_SCHEMA);
        assert_eq!(value["dry_run"], true);
        assert!(!String::from_utf8_lossy(&bytes).contains(root.to_string_lossy().as_ref()));
        assert!(
            !root
                .join(format!(".aetherion-plugin-run-{}.tmp", std::process::id()))
                .exists()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dry_run_report_matches_the_versioned_golden_shape() {
        let root = directory("dry-run-golden");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("plugin.json");
        let module_path = root.join("plugin.wasm");
        fs::write(&manifest_path, serde_json::to_vec(&manifest()).unwrap()).unwrap();
        fs::write(&module_path, RETURN_SEVEN).unwrap();
        let report = run(RunOptions {
            manifest: manifest_path,
            module: module_path,
            path: None,
            scene: None,
            assets: None,
            export: DEFAULT_ENTRYPOINT.into(),
            dry_run: true,
            report: None,
        })
        .unwrap();
        let golden =
            include_str!("../tests/fixtures/plugin-run-dry-run-v1.json").replace("\r\n", "\n");
        assert_eq!(
            serde_json::to_string_pretty(&report).unwrap(),
            golden.trim_end()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn execution_report_is_atomic_and_contains_no_machine_paths() {
        let root = directory("execute");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("plugin.json");
        let module_path = root.join("plugin.wasm");
        let report_path = root.join("report.json");
        fs::write(&manifest_path, serde_json::to_vec(&manifest()).unwrap()).unwrap();
        fs::write(&module_path, RETURN_SEVEN).unwrap();
        let report = run(RunOptions {
            manifest: manifest_path,
            module: module_path,
            path: None,
            scene: None,
            assets: None,
            export: DEFAULT_ENTRYPOINT.into(),
            dry_run: false,
            report: Some(report_path.clone()),
        })
        .unwrap();
        assert_eq!(report.status, "executed");
        assert_eq!(report.return_code, Some(7));
        let golden =
            include_str!("../tests/fixtures/plugin-run-executed-v1.json").replace("\r\n", "\n");
        assert_eq!(
            serde_json::to_string_pretty(&report).unwrap(),
            golden.trim_end()
        );
        let bytes = fs::read(report_path).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains(root.to_string_lossy().as_ref()));
        let report_again = run(RunOptions {
            manifest: root.join("plugin.json"),
            module: root.join("plugin.wasm"),
            path: None,
            scene: None,
            assets: None,
            export: DEFAULT_ENTRYPOINT.into(),
            dry_run: false,
            report: Some(root.join("report.json")),
        })
        .unwrap();
        assert_eq!(report_again, report);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn execution_reports_with_telemetry_and_io_match_the_golden_contract() {
        let root = directory("telemetry-io-golden");
        fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("plugin.json");
        let module_path = root.join("plugin.wasm");
        let assets_path = root.join("assets.json");
        let asset_path = root.join("payload.bin");
        fs::write(
            &manifest_path,
            serde_json::to_vec(&manifest_with_capabilities(
                vec![
                    crate::plugin::Capability::AssetRead,
                    crate::plugin::Capability::TelemetryWrite,
                ],
                1,
                1,
            ))
            .unwrap(),
        )
        .unwrap();
        fs::write(&asset_path, [7_u8]).unwrap();
        fs::write(
            &assets_path,
            format!(
                r#"{{"schema":"aetherion.assets/v1","assets":[{{"id":"payload","path":"payload.bin","type":"pam-rgba","size":1,"checksum":{}}}]}}"#,
                crate::render::checksum_bytes(&[7])
            ),
        )
        .unwrap();

        fs::write(&module_path, telemetry_module()).unwrap();
        let telemetry = run(RunOptions {
            manifest: manifest_path.clone(),
            module: module_path.clone(),
            path: None,
            scene: None,
            assets: Some(assets_path.clone()),
            export: DEFAULT_ENTRYPOINT.into(),
            dry_run: false,
            report: None,
        })
        .unwrap();
        let telemetry_golden =
            include_str!("../tests/fixtures/plugin-run-telemetry-v1.json").replace("\r\n", "\n");
        assert_eq!(
            serde_json::to_string_pretty(&telemetry).unwrap(),
            telemetry_golden.trim_end()
        );

        fs::write(&module_path, asset_module()).unwrap();
        let io = run(RunOptions {
            manifest: manifest_path,
            module: module_path,
            path: None,
            scene: None,
            assets: Some(assets_path),
            export: DEFAULT_ENTRYPOINT.into(),
            dry_run: false,
            report: None,
        })
        .unwrap();
        let io_golden =
            include_str!("../tests/fixtures/plugin-run-io-v1.json").replace("\r\n", "\n");
        assert_eq!(
            serde_json::to_string_pretty(&io).unwrap(),
            io_golden.trim_end()
        );
        fs::remove_dir_all(root).unwrap();
    }

    fn manifest_with_capabilities(
        capabilities: Vec<crate::plugin::Capability>,
        io_read_bytes: u64,
        files: u32,
    ) -> PluginManifest {
        let mut manifest = manifest();
        manifest.capabilities = capabilities;
        manifest.quotas.io_read_bytes = io_read_bytes;
        manifest.quotas.files = files;
        manifest
    }

    fn imported_module(import_name: &str, function_type: &[u8], body: &[u8]) -> Vec<u8> {
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
        string(&mut imports, crate::plugin_runtime::HOST_MODULE);
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

    fn telemetry_module() -> Vec<u8> {
        imported_module(
            "telemetry_write",
            &[0x60, 2, 0x7e, 0x7e, 1, 0x7f],
            &[0x42, 7, 0x42, 9, 0x10, 0x00, 0x1a, 0x41, 0],
        )
    }

    fn asset_module() -> Vec<u8> {
        imported_module(
            "asset_read_byte",
            &[0x60, 2, 0x7f, 0x7f, 1, 0x7f],
            &[0x41, 0, 0x41, 0, 0x10, 0x00],
        )
    }
}
