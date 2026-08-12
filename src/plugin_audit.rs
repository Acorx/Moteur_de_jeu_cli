use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::Result;
use crate::plugin::{self, Capability, HOST_ABI_MAJOR, HOST_ABI_MINOR, HostAbi, PluginQuotas};
use crate::plugin_runtime::{self, DEFAULT_ENTRYPOINT, HostContext};
use crate::render::checksum_bytes;

pub const REPORT_SCHEMA: &str = "aetherion.plugin-audit/v1";
pub const RUNTIME_ENGINE: &str = "wasmi";
pub const RUNTIME_VERSION: &str = "0.32.3";

#[derive(Clone, Debug)]
pub struct AuditOptions {
    pub manifest: PathBuf,
    pub module: PathBuf,
    pub export: String,
    pub report: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PluginAuditReport {
    pub schema: &'static str,
    pub status: &'static str,
    pub plugin_id: String,
    pub plugin_version: String,
    pub export: String,
    pub manifest_checksum_fnv1a: u64,
    pub module_checksum_fnv1a: u64,
    pub host_abi: HostAbi,
    pub runtime: RuntimeInfo,
    pub capabilities: Vec<Capability>,
    pub quotas: PluginQuotas,
    pub signatures: SignatureInfo,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RuntimeInfo {
    pub engine: &'static str,
    pub version: &'static str,
    pub network: bool,
    pub wasi: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SignatureInfo {
    pub status: &'static str,
}

pub fn audit(options: AuditOptions) -> Result<PluginAuditReport> {
    let manifest_bytes = read_regular_file(&options.manifest, "plugin_audit_manifest")?;
    let module_bytes = read_regular_file(&options.module, "plugin_audit_module")?;
    let manifest = plugin::load(&options.manifest)?;
    let export = normalized_export(&options.export)?;

    plugin_runtime::validate_bytes_with_manifest(
        &module_bytes,
        &export,
        &manifest,
        &HostContext::default(),
    )?;

    let report = PluginAuditReport {
        schema: REPORT_SCHEMA,
        status: "verified",
        plugin_id: manifest.id,
        plugin_version: manifest.version,
        export,
        manifest_checksum_fnv1a: checksum_bytes(&manifest_bytes),
        module_checksum_fnv1a: checksum_bytes(&module_bytes),
        host_abi: HostAbi {
            major: HOST_ABI_MAJOR,
            minor: HOST_ABI_MINOR,
        },
        runtime: RuntimeInfo {
            engine: RUNTIME_ENGINE,
            version: RUNTIME_VERSION,
            network: false,
            wasi: false,
        },
        capabilities: manifest.capabilities,
        quotas: manifest.quotas,
        signatures: SignatureInfo {
            status: "not_implemented",
        },
    };
    if let Some(path) = options.report.as_deref() {
        write_atomic_report(path, &report)?;
    }
    Ok(report)
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

fn read_regular_file(path: &Path, prefix: &str) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{prefix}_read: {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("{prefix}_not_regular_file").into());
    }
    let bytes =
        fs::read(path).map_err(|error| format!("{prefix}_read: {}: {error}", path.display()))?;
    if bytes.is_empty() {
        return Err(if prefix == "plugin_audit_module" {
            "plugin_runtime_module_empty".into()
        } else {
            "plugin_manifest_invalid: manifeste vide".into()
        });
    }
    Ok(bytes)
}

fn write_atomic_report(path: &Path, report: &PluginAuditReport) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| format!("plugin_audit_report_serialize: {error}"))?;
    bytes.push(b'\n');
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("plugin_audit_report_mkdir: {}: {error}", parent.display()))?;
    let temporary = parent.join(format!(
        ".aetherion-plugin-audit-{}.tmp",
        std::process::id()
    ));
    fs::write(&temporary, bytes).map_err(|error| {
        format!(
            "plugin_audit_report_write: {}: {error}",
            temporary.display()
        )
    })?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("plugin_audit_report_replace: {}: {error}", path.display()))?;
    }
    fs::rename(&temporary, path)
        .map_err(|error| format!("plugin_audit_report_rename: {}: {error}", path.display()).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_provenance_is_explicitly_sandboxed() {
        assert_eq!(REPORT_SCHEMA, "aetherion.plugin-audit/v1");
        assert_eq!(RUNTIME_ENGINE, "wasmi");
        assert_eq!(RUNTIME_VERSION, "0.32.3");
        assert!(
            !RuntimeInfo {
                engine: RUNTIME_ENGINE,
                version: RUNTIME_VERSION,
                network: false,
                wasi: false,
            }
            .network
        );
    }
}
