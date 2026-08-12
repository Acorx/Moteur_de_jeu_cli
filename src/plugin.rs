use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;

pub const MANIFEST_SCHEMA: &str = "aetherion.plugin/v1";
pub const HOST_ABI_MAJOR: u32 = 1;
pub const HOST_ABI_MINOR: u32 = 1;
pub const PREVIOUS_HOST_ABI_MINOR: u32 = 0;
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
pub const MAX_PLUGINS: usize = 256;
pub const MAX_MEMORY_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_FUEL: u64 = 1_000_000_000;
pub const MAX_IO_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_FILES: u32 = 1024;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    pub schema: String,
    pub id: String,
    pub version: String,
    pub abi: PluginAbi,
    pub capabilities: Vec<Capability>,
    pub quotas: PluginQuotas,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PluginAbi {
    pub major: u32,
    pub minimum_host_minor: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PluginQuotas {
    pub memory_bytes: u64,
    pub fuel: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
    pub files: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    AssetRead,
    SceneRead,
    SimulationRead,
    TelemetryWrite,
}

#[derive(Debug, Serialize)]
pub struct CatalogEntry {
    pub id: String,
    pub version: String,
    pub path: String,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Serialize)]
pub struct Catalog {
    pub schema: &'static str,
    pub abi: HostAbi,
    pub plugins: Vec<CatalogEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct HostAbi {
    pub major: u32,
    pub minor: u32,
}

pub fn load(path: &Path) -> Result<PluginManifest> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("plugin_manifest_read: {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("plugin_manifest_not_regular_file".into());
    }
    if metadata.len() > MAX_MANIFEST_BYTES {
        return Err("plugin_manifest_too_large".into());
    }
    let bytes = fs::read(path)
        .map_err(|error| format!("plugin_manifest_read: {}: {error}", path.display()))?;
    let mut manifest: PluginManifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("plugin_manifest_invalid: {error}"))?;
    validate(&manifest)?;
    manifest.capabilities.sort();
    Ok(manifest)
}

pub fn validate(manifest: &PluginManifest) -> Result<()> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!("plugin_manifest_version: attendu {MANIFEST_SCHEMA}").into());
    }
    validate_id(&manifest.id)?;
    validate_version(&manifest.version)?;
    validate_against_host(
        manifest,
        HostAbi {
            major: HOST_ABI_MAJOR,
            minor: HOST_ABI_MINOR,
        },
    )?;
    let unique: BTreeSet<_> = manifest.capabilities.iter().copied().collect();
    if unique.len() != manifest.capabilities.len() {
        return Err("plugin_capability_duplicate".into());
    }
    validate_quotas(&manifest.quotas)
}

pub fn validate_against_host(manifest: &PluginManifest, host: HostAbi) -> Result<()> {
    if manifest.abi.major != host.major || manifest.abi.minimum_host_minor > host.minor {
        return Err(format!(
            "plugin_abi_incompatible: plugin {}.{}; hôte {}.{}",
            manifest.abi.major, manifest.abi.minimum_host_minor, host.major, host.minor
        )
        .into());
    }
    Ok(())
}

pub fn inspect(path: &Path) -> Result<String> {
    serde_json::to_string_pretty(&load(path)?)
        .map_err(|error| format!("plugin_manifest_serialize: {error}").into())
}

pub fn validation_report(path: &Path) -> Result<String> {
    let manifest = load(path)?;
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "aetherion.plugin-validation/v1",
        "status": "valid",
        "id": manifest.id,
        "version": manifest.version,
        "host_abi": {"major": HOST_ABI_MAJOR, "minor": HOST_ABI_MINOR},
        "compatibility": {
            "policy": "same_major_and_minimum_host_minor_at_most_host_minor",
            "previous_host_minor": PREVIOUS_HOST_ABI_MINOR
        }
    }))
    .map_err(|error| format!("plugin_validation_serialize: {error}").into())
}

pub fn load_catalog(root: &Path) -> Result<Catalog> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| format!("plugin_root_read: {}: {error}", root.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("plugin_root_not_directory".into());
    }
    let mut paths = Vec::<PathBuf>::new();
    for entry in fs::read_dir(root)
        .map_err(|error| format!("plugin_root_read: {}: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| format!("plugin_root_entry: {error}"))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".plugin.json") {
            paths.push(entry.path());
        }
    }
    paths.sort_by(|left, right| {
        left.file_name()
            .unwrap_or_default()
            .cmp(right.file_name().unwrap_or_default())
    });
    if paths.len() > MAX_PLUGINS {
        return Err("plugin_count_quota".into());
    }

    let mut by_id = BTreeMap::new();
    for path in paths {
        let manifest = load(&path)?;
        let relative = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let entry = CatalogEntry {
            id: manifest.id.clone(),
            version: manifest.version,
            path: relative,
            capabilities: manifest.capabilities,
        };
        if by_id.insert(manifest.id, entry).is_some() {
            return Err("plugin_duplicate_id".into());
        }
    }
    Ok(Catalog {
        schema: "aetherion.plugin-catalog/v1",
        abi: HostAbi {
            major: HOST_ABI_MAJOR,
            minor: HOST_ABI_MINOR,
        },
        plugins: by_id.into_values().collect(),
    })
}

pub fn catalog_json(root: &Path) -> Result<String> {
    serde_json::to_string_pretty(&load_catalog(root)?)
        .map_err(|error| format!("plugin_catalog_serialize: {error}").into())
}

fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 64
        || !id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
        || !id.as_bytes()[0].is_ascii_lowercase()
    {
        return Err(format!("plugin_id_invalid: {id}").into());
    }
    Ok(())
}

fn validate_version(version: &str) -> Result<()> {
    let parts: Vec<_> = version.split('.').collect();
    if parts.len() != 3
        || parts.iter().any(|part| {
            part.is_empty()
                || !part.bytes().all(|byte| byte.is_ascii_digit())
                || (part.len() > 1 && part.starts_with('0'))
                || part.parse::<u32>().is_err()
        })
    {
        return Err(format!("plugin_version_invalid: {version}").into());
    }
    Ok(())
}

fn validate_quotas(quotas: &PluginQuotas) -> Result<()> {
    if quotas.memory_bytes == 0 || quotas.memory_bytes > MAX_MEMORY_BYTES {
        return Err("plugin_quota_memory".into());
    }
    if quotas.fuel == 0 || quotas.fuel > MAX_FUEL {
        return Err("plugin_quota_fuel".into());
    }
    if quotas.io_read_bytes > MAX_IO_BYTES || quotas.io_write_bytes > MAX_IO_BYTES {
        return Err("plugin_quota_io".into());
    }
    if quotas.files > MAX_FILES {
        return Err("plugin_quota_files".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> PluginManifest {
        PluginManifest {
            schema: MANIFEST_SCHEMA.into(),
            id: "org.aetherion.example".into(),
            version: "1.2.3".into(),
            abi: PluginAbi {
                major: HOST_ABI_MAJOR,
                minimum_host_minor: HOST_ABI_MINOR,
            },
            capabilities: vec![Capability::TelemetryWrite, Capability::AssetRead],
            quotas: PluginQuotas {
                memory_bytes: 1024,
                fuel: 1000,
                io_read_bytes: 1024,
                io_write_bytes: 1024,
                files: 2,
            },
        }
    }

    #[test]
    fn strict_manifest_is_validated_and_canonicalized() {
        let directory =
            std::env::temp_dir().join(format!("aetherion-plugin-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("example.plugin.json");
        fs::write(&path, serde_json::to_vec(&valid()).unwrap()).unwrap();
        let loaded = load(&path).unwrap();
        assert_eq!(
            loaded.capabilities,
            [Capability::AssetRead, Capability::TelemetryWrite]
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn minor_abi_compatibility_is_stable_across_host_1_0_and_1_1() {
        let mut minimum_zero = valid();
        minimum_zero.abi.minimum_host_minor = 0;
        assert!(validate_against_host(&minimum_zero, HostAbi { major: 1, minor: 0 }).is_ok());
        assert!(validate_against_host(&minimum_zero, HostAbi { major: 1, minor: 1 }).is_ok());

        let minimum_one = valid();
        assert!(validate_against_host(&minimum_one, HostAbi { major: 1, minor: 1 }).is_ok());
        assert!(
            validate_against_host(&minimum_one, HostAbi { major: 1, minor: 0 })
                .unwrap_err()
                .to_string()
                .contains("plugin_abi_incompatible")
        );

        let mut incompatible_major = valid();
        incompatible_major.abi.major = 2;
        assert!(
            validate_against_host(&incompatible_major, HostAbi { major: 1, minor: 1 }).is_err()
        );
    }

    #[test]
    fn incompatible_abi_duplicates_and_quotas_are_rejected() {
        let mut manifest = valid();
        manifest.abi.major += 1;
        assert!(
            validate(&manifest)
                .unwrap_err()
                .to_string()
                .contains("plugin_abi_incompatible")
        );
        manifest = valid();
        manifest.capabilities.push(Capability::AssetRead);
        assert!(
            validate(&manifest)
                .unwrap_err()
                .to_string()
                .contains("plugin_capability_duplicate")
        );
        manifest = valid();
        manifest.quotas.memory_bytes = MAX_MEMORY_BYTES + 1;
        assert!(
            validate(&manifest)
                .unwrap_err()
                .to_string()
                .contains("plugin_quota_memory")
        );
    }

    #[test]
    fn catalog_is_sorted_by_id_and_rejects_duplicates() {
        let directory =
            std::env::temp_dir().join(format!("aetherion-plugin-catalog-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let mut z = valid();
        z.id = "org.example.z".into();
        let mut a = valid();
        a.id = "org.example.a".into();
        fs::write(
            directory.join("1.plugin.json"),
            serde_json::to_vec(&z).unwrap(),
        )
        .unwrap();
        fs::write(
            directory.join("2.plugin.json"),
            serde_json::to_vec(&a).unwrap(),
        )
        .unwrap();
        let catalog = load_catalog(&directory).unwrap();
        assert_eq!(catalog.plugins[0].id, "org.example.a");
        assert_eq!(catalog.plugins[1].id, "org.example.z");
        a.id = z.id;
        fs::write(
            directory.join("2.plugin.json"),
            serde_json::to_vec(&a).unwrap(),
        )
        .unwrap();
        assert!(
            load_catalog(&directory)
                .unwrap_err()
                .to_string()
                .contains("plugin_duplicate_id")
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
