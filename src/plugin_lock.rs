use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::error::AppError;
use crate::plugin::{self, Capability};
use crate::render::checksum_bytes;

pub const LOCK_SCHEMA: &str = "aetherion.plugin-lock/v1";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PluginLock {
    pub schema: String,
    pub plugins: Vec<PluginLockEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PluginLockEntry {
    pub id: String,
    pub path: String,
    pub abi_version: u32,
    pub checksum_fnv1a: u64,
    pub capabilities: Vec<Capability>,
    pub version: String,
}

pub fn resolve(dir: &Path, lockfile: &Path) -> Result<PluginLock> {
    let mut plugins = entries(dir)?;
    plugins.sort_by(|a, b| a.id.cmp(&b.id));
    let lock = PluginLock {
        schema: LOCK_SCHEMA.into(),
        plugins,
    };
    write_atomic(
        lockfile,
        &serde_json::to_vec_pretty(&lock).map_err(|e| format!("plugin_lock_serialize: {e}"))?,
    )?;
    Ok(lock)
}

pub fn check(dir: &Path, lockfile: &Path) -> Result<PluginLock> {
    let bytes =
        fs::read(lockfile).map_err(|e| format!("plugin_lock_read: {}: {e}", lockfile.display()))?;
    let mut lock: PluginLock =
        serde_json::from_slice(&bytes).map_err(|e| format!("plugin_lock_invalid: {e}"))?;
    if lock.schema != LOCK_SCHEMA {
        return Err("plugin_lock_version".into());
    }
    lock.plugins.sort_by(|a, b| a.id.cmp(&b.id));
    if lock.plugins.windows(2).any(|p| p[0].id == p[1].id) {
        return Err("plugin_lock_duplicate_id".into());
    }
    let mut actual = entries(dir)?;
    actual.sort_by(|a, b| a.id.cmp(&b.id));
    if lock.plugins != actual {
        let json = serde_json::to_string_pretty(&serde_json::json!({"schema":"aetherion.plugin-lock-check/v1","status":"diverged","expected":lock.plugins,"actual":actual})).map_err(|e| format!("plugin_lock_serialize: {e}"))?;
        return Err(AppError::outcome("plugin_lock_diverged", 1, json));
    }
    Ok(lock)
}

fn entries(dir: &Path) -> Result<Vec<PluginLockEntry>> {
    let catalog = plugin::load_catalog(dir)?;
    catalog
        .plugins
        .into_iter()
        .map(|entry| {
            let path = dir.join(&entry.path);
            let bytes = fs::read(&path)
                .map_err(|e| format!("plugin_lock_read: {}: {e}", path.display()))?;
            let manifest = plugin::load(&path)?;
            Ok(PluginLockEntry {
                id: entry.id,
                path: entry.path.replace('\\', "/"),
                abi_version: manifest.abi.major,
                checksum_fnv1a: checksum_bytes(&bytes),
                capabilities: entry.capabilities,
                version: entry.version,
            })
        })
        .collect()
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|e| format!("plugin_lock_mkdir: {}: {e}", parent.display()))?;
    }
    let mut tmp = PathBuf::from(path);
    tmp.set_extension(format!("tmp-{}", std::process::id()));
    let mut output = bytes.to_vec();
    output.push(b'\n');
    fs::write(&tmp, output).map_err(|e| format!("plugin_lock_write: {}: {e}", tmp.display()))?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|e| format!("plugin_lock_replace: {}: {e}", path.display()))?;
    }
    fs::rename(&tmp, path)
        .map_err(|e| format!("plugin_lock_rename: {}: {e}", path.display()).into())
}
