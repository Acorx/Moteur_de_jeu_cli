use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::Result;

pub const TELEMETRY_SCHEMA: &str = "aetherion.telemetry/v1";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SystemCounters {
    pub name: &'static str,
    pub ticks: u64,
    pub entities_visited: u64,
    pub entities_modified: u64,
    pub events_applied: u64,
    pub prng_calls: u64,
    pub collisions_resolved: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Telemetry {
    pub schema: &'static str,
    pub tick: u64,
    pub checksum: u64,
    pub system_order: Vec<&'static str>,
    pub systems: Vec<SystemCounters>,
}

pub fn save(telemetry: &Telemetry, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|error| format!("création de {}: {error}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(telemetry)
        .map_err(|error| format!("sérialisation de la télémétrie: {error}"))?;
    fs::write(path, [bytes.as_slice(), b"\n"].concat())
        .map_err(|error| format!("écriture de {}: {error}", path.display()).into())
}
