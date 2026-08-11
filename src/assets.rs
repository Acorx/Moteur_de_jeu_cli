use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::render::checksum_bytes;

pub const MANIFEST_FILE: &str = "assets.json";
pub const MAX_MANIFEST_BYTES: u64 = 1_048_576;
pub const DEFAULT_MAX_ASSETS: usize = 256;
pub const DEFAULT_MAX_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetManifest {
    pub schema: String,
    #[serde(default)]
    pub budgets: AssetBudgets,
    pub assets: Vec<AssetEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetBudgets {
    #[serde(default = "default_count")]
    pub max_count: usize,
    #[serde(default = "default_bytes")]
    pub max_bytes: u64,
}

impl Default for AssetBudgets {
    fn default() -> Self {
        Self {
            max_count: default_count(),
            max_bytes: default_bytes(),
        }
    }
}
const fn default_count() -> usize {
    DEFAULT_MAX_ASSETS
}
const fn default_bytes() -> u64 {
    DEFAULT_MAX_BYTES
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetEntry {
    pub id: String,
    pub path: String,
    #[serde(rename = "type")]
    pub kind: AssetType,
    pub size: u64,
    pub checksum: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AssetType {
    PamRgba,
}

#[derive(Clone, Debug)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssetInventory {
    pub schema: &'static str,
    pub manifest: String,
    pub budgets: AssetBudgets,
    pub declared_count: usize,
    pub loaded_count: usize,
    pub loaded_bytes: u64,
    pub cache_hits: u64,
    pub assets: Vec<AssetStatus>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssetStatus {
    pub id: String,
    pub path: String,
    #[serde(rename = "type")]
    pub kind: AssetType,
    pub expected_size: u64,
    pub expected_checksum: u64,
    pub loaded: bool,
}

#[derive(Clone, Debug)]
pub struct AssetManager {
    root: PathBuf,
    manifest_path: PathBuf,
    manifest: AssetManifest,
    entries: BTreeMap<String, AssetEntry>,
    textures: BTreeMap<String, Texture>,
    loaded_bytes: u64,
    cache_hits: u64,
}

impl AssetManager {
    pub fn load(root: &Path, manifest: Option<&Path>) -> Result<Self> {
        let root = root
            .canonicalize()
            .map_err(|e| format!("asset_root_invalid: {}: {e}", root.display()))?;
        let requested = manifest.unwrap_or_else(|| Path::new(MANIFEST_FILE));
        let manifest_path = confined(&root, requested, false)?;
        let metadata = fs::metadata(&manifest_path)
            .map_err(|e| format!("asset_manifest_read: {}: {e}", manifest_path.display()))?;
        if metadata.len() > MAX_MANIFEST_BYTES {
            return Err("asset_manifest_too_large: plafond 1048576 octets".into());
        }
        let bytes = fs::read(&manifest_path)
            .map_err(|e| format!("asset_manifest_read: {}: {e}", manifest_path.display()))?;
        let value: AssetManifest =
            serde_json::from_slice(&bytes).map_err(|e| format!("asset_manifest_invalid: {e}"))?;
        if value.schema != "aetherion.assets/v1" {
            return Err("asset_manifest_version: attendu aetherion.assets/v1".into());
        }
        if value.budgets.max_count == 0
            || value.budgets.max_count > DEFAULT_MAX_ASSETS
            || value.budgets.max_bytes == 0
            || value.budgets.max_bytes > DEFAULT_MAX_BYTES
        {
            return Err("asset_budget_invalid: plafonds internes dépassés".into());
        }
        if value.assets.len() > value.budgets.max_count {
            return Err("asset_count_quota: manifeste au-dessus de max_count".into());
        }
        let mut entries = BTreeMap::new();
        let mut declared = 0_u64;
        for entry in &value.assets {
            validate_id(&entry.id)?;
            if entries.contains_key(&entry.id) {
                return Err(format!("asset_duplicate_id: {}", entry.id).into());
            }
            confined(&root, Path::new(&entry.path), false)?;
            declared = declared
                .checked_add(entry.size)
                .ok_or("asset_bytes_quota: dépassement")?;
            if declared > value.budgets.max_bytes {
                return Err("asset_bytes_quota: tailles déclarées au-dessus de max_bytes".into());
            }
            entries.insert(entry.id.clone(), entry.clone());
        }
        Ok(Self {
            root,
            manifest_path,
            manifest: value,
            entries,
            textures: BTreeMap::new(),
            loaded_bytes: 0,
            cache_hits: 0,
        })
    }

    pub fn prepare(&mut self, ids: impl IntoIterator<Item = String>) -> Result<()> {
        let mut ids: Vec<String> = ids.into_iter().collect();
        ids.sort();
        ids.dedup();
        for id in ids {
            self.texture(&id)?;
        }
        Ok(())
    }

    pub fn prepare_concurrent(
        &mut self,
        ids: impl IntoIterator<Item = String>,
    ) -> Result<BTreeMap<String, Texture>> {
        let mut ids: Vec<String> = ids.into_iter().collect();
        ids.sort();
        ids.dedup();
        let mut jobs = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(texture) = self.textures.get(&id) {
                self.cache_hits += 1;
                jobs.push((id, None, Some(texture.clone())));
                continue;
            }
            let entry = self
                .entries
                .get(&id)
                .ok_or_else(|| format!("asset_unknown: {id}"))?
                .clone();
            let path = confined(&self.root, Path::new(&entry.path), true)?;
            jobs.push((id, Some((entry, path)), None));
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            for (id, job, cached) in jobs {
                let sender = sender.clone();
                scope.spawn(move || {
                    let result = if let Some(texture) = cached {
                        Ok((texture, 0))
                    } else {
                        let (entry, path) = job.expect("uncached asset has a load job");
                        load_texture(&id, &entry, &path).map(|texture| (texture, entry.size))
                    };
                    let _ = sender.send((id, result));
                });
            }
        });
        drop(sender);
        let mut loaded = BTreeMap::new();
        let mut added_bytes = 0_u64;
        for (id, result) in receiver {
            let (texture, size) = result?;
            added_bytes = added_bytes
                .checked_add(size)
                .ok_or("asset_bytes_quota: dépassement")?;
            loaded.insert(id, texture);
        }
        let next = self
            .loaded_bytes
            .checked_add(added_bytes)
            .ok_or("asset_bytes_quota: dépassement")?;
        if next > self.manifest.budgets.max_bytes {
            return Err("asset_bytes_quota: cache au-dessus de max_bytes".into());
        }
        self.loaded_bytes = next;
        self.textures.extend(loaded.clone());
        Ok(loaded)
    }

    pub fn resolved(
        &self,
        ids: impl IntoIterator<Item = String>,
    ) -> Result<BTreeMap<String, Texture>> {
        let mut result = BTreeMap::new();
        for id in ids {
            let texture = self
                .textures
                .get(&id)
                .ok_or_else(|| format!("asset_not_loaded: {id}"))?;
            result.insert(id, texture.clone());
        }
        Ok(result)
    }

    pub fn texture(&mut self, id: &str) -> Result<&Texture> {
        if self.textures.contains_key(id) {
            self.cache_hits += 1;
            return Ok(&self.textures[id]);
        }
        let entry = self
            .entries
            .get(id)
            .ok_or_else(|| format!("asset_unknown: {id}"))?
            .clone();
        let path = confined(&self.root, Path::new(&entry.path), true)?;
        let bytes = fs::read(&path).map_err(|e| format!("asset_read: {}: {e}", path.display()))?;
        if bytes.len() as u64 != entry.size {
            return Err(format!(
                "asset_size_mismatch: {id}: attendu {}, obtenu {}",
                entry.size,
                bytes.len()
            )
            .into());
        }
        let actual = checksum_bytes(&bytes);
        if actual != entry.checksum {
            return Err(format!(
                "asset_checksum_mismatch: {id}: attendu {}, obtenu {actual}",
                entry.checksum
            )
            .into());
        }
        let next = self
            .loaded_bytes
            .checked_add(entry.size)
            .ok_or("asset_bytes_quota: dépassement")?;
        if next > self.manifest.budgets.max_bytes {
            return Err("asset_bytes_quota: cache au-dessus de max_bytes".into());
        }
        let texture = match entry.kind {
            AssetType::PamRgba => decode_pam_rgba(&bytes)?,
        };
        self.loaded_bytes = next;
        self.textures.insert(id.to_owned(), texture);
        Ok(&self.textures[id])
    }

    pub fn inventory(&self) -> AssetInventory {
        AssetInventory {
            schema: "aetherion.asset-inventory/v1",
            manifest: self.manifest_path.to_string_lossy().replace('\\', "/"),
            budgets: self.manifest.budgets.clone(),
            declared_count: self.entries.len(),
            loaded_count: self.textures.len(),
            loaded_bytes: self.loaded_bytes,
            cache_hits: self.cache_hits,
            assets: self
                .entries
                .values()
                .map(|e| AssetStatus {
                    id: e.id.clone(),
                    path: e.path.clone(),
                    kind: e.kind,
                    expected_size: e.size,
                    expected_checksum: e.checksum,
                    loaded: self.textures.contains_key(&e.id),
                })
                .collect(),
        }
    }
}

fn load_texture(id: &str, entry: &AssetEntry, path: &Path) -> Result<Texture> {
    let bytes = fs::read(path).map_err(|e| format!("asset_read: {}: {e}", path.display()))?;
    if bytes.len() as u64 != entry.size {
        return Err(format!(
            "asset_size_mismatch: {id}: attendu {}, obtenu {}",
            entry.size,
            bytes.len()
        )
        .into());
    }
    let actual = checksum_bytes(&bytes);
    if actual != entry.checksum {
        return Err(format!(
            "asset_checksum_mismatch: {id}: attendu {}, obtenu {actual}",
            entry.checksum
        )
        .into());
    }
    match entry.kind {
        AssetType::PamRgba => decode_pam_rgba(&bytes),
    }
}

fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    {
        return Err(format!("asset_id_invalid: {id}").into());
    }
    Ok(())
}

fn confined(root: &Path, requested: &Path, must_exist: bool) -> Result<PathBuf> {
    if requested.is_absolute()
        || requested.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err("asset_path_traversal: chemin relatif confiné requis".into());
    }
    let candidate = root.join(requested);
    if must_exist {
        let canonical = candidate
            .canonicalize()
            .map_err(|e| format!("asset_path_invalid: {}: {e}", candidate.display()))?;
        if !canonical.starts_with(root) {
            return Err("asset_path_traversal: sortie de la racine".into());
        }
        Ok(canonical)
    } else {
        Ok(candidate)
    }
}

pub fn decode_pam_rgba(bytes: &[u8]) -> Result<Texture> {
    const END: &[u8] = b"ENDHDR\n";
    let end = bytes
        .windows(END.len())
        .position(|w| w == END)
        .ok_or("pam_invalid: ENDHDR absent")?
        + END.len();
    let header =
        std::str::from_utf8(&bytes[..end]).map_err(|_| "pam_invalid: en-tête non UTF-8")?;
    let mut width = None;
    let mut height = None;
    let mut depth = None;
    let mut maxval = None;
    let mut tuple = None;
    for line in header.lines().skip(1) {
        let mut parts = line.split_whitespace();
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");
        match key {
            "WIDTH" => width = value.parse().ok(),
            "HEIGHT" => height = value.parse().ok(),
            "DEPTH" => depth = value.parse().ok(),
            "MAXVAL" => maxval = value.parse().ok(),
            "TUPLTYPE" => tuple = Some(value),
            _ => {}
        }
    }
    let (width, height) = (
        width.ok_or("pam_invalid: WIDTH")?,
        height.ok_or("pam_invalid: HEIGHT")?,
    );
    if !bytes.starts_with(b"P7\n")
        || depth != Some(4)
        || maxval != Some(255)
        || tuple != Some("RGB_ALPHA")
        || width == 0
        || height == 0
        || width > 8192
        || height > 8192
    {
        return Err("pam_invalid: P7 RGBA 8-bit requis".into());
    }
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|v| v.checked_mul(4))
        .ok_or("pam_invalid: dimensions")?;
    if bytes.len() - end != expected {
        return Err("pam_invalid: taille pixels incohérente".into());
    }
    Ok(Texture {
        width,
        height,
        rgba: bytes[end..].to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pam_decodes() {
        let mut b =
            b"P7\nWIDTH 1\nHEIGHT 1\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n".to_vec();
        b.extend([1, 2, 3, 4]);
        let t = decode_pam_rgba(&b).unwrap();
        assert_eq!(t.rgba, [1, 2, 3, 4]);
    }
    #[test]
    fn synchronous_and_concurrent_loading_are_identical() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let mut synchronous = AssetManager::load(&root, Some(Path::new("assets.json"))).unwrap();
        synchronous.prepare(vec!["checker".to_string()]).unwrap();
        let sync = synchronous.resolved(vec!["checker".to_string()]).unwrap();
        let mut concurrent = AssetManager::load(&root, Some(Path::new("assets.json"))).unwrap();
        let parallel = concurrent
            .prepare_concurrent(vec!["checker".to_string()])
            .unwrap();
        assert_eq!(sync["checker"].width, parallel["checker"].width);
        assert_eq!(sync["checker"].height, parallel["checker"].height);
        assert_eq!(sync["checker"].rgba, parallel["checker"].rgba);
    }

    #[test]
    fn traversal_is_rejected() {
        let root = std::env::current_dir().unwrap().canonicalize().unwrap();
        assert!(confined(&root, Path::new("../x"), false).is_err());
    }
}
