use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::render::checksum_bytes;
use crate::render3d::{Material3d, Mesh3d};

pub const MANIFEST_SCHEMA: &str = "aetherion.assets3d/v1";
pub const MESH_SCHEMA: &str = "aetherion.mesh3d/v1";
pub const MATERIAL_SCHEMA: &str = "aetherion.material3d/v1";
pub const MAX_TEXTURE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_MANIFEST_BYTES: u64 = 1_048_576;
pub const MAX_ASSETS: usize = 20_000;
pub const MAX_ASSET_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 64 * 1024 * 1024;
pub const CACHE_FORMAT_VERSION: &str = "format-v1";
pub const IMPORTER_VERSION: &str = "assets3d-loader-v1";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Asset3dManifest {
    pub schema: String,
    pub assets: Vec<Asset3dEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Asset3dEntry {
    pub id: String,
    pub path: String,
    #[serde(rename = "type")]
    pub kind: Asset3dType,
    pub size: u64,
    pub checksum: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Asset3dType {
    Mesh,
    Material,
    Texture,
}

#[derive(Clone, Debug)]
pub enum Asset3d {
    Mesh(Mesh3d),
    Material(Material3d),
    Texture(Texture3d),
}

#[derive(Clone, Debug)]
pub struct Texture3d {
    pub id: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MeshDocument {
    schema: String,
    mesh: Mesh3d,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MaterialDocument {
    schema: String,
    material: Material3d,
}

pub fn load_manifest(path: &Path) -> Result<BTreeMap<String, Asset3d>> {
    load_manifest_internal(path, None)
}

pub fn load_manifest_cached(path: &Path, cache_root: &Path) -> Result<BTreeMap<String, Asset3d>> {
    load_manifest_internal(path, Some(cache_root))
}

fn load_manifest_internal(
    path: &Path,
    cache_root: Option<&Path>,
) -> Result<BTreeMap<String, Asset3d>> {
    let metadata = fs::metadata(path)
        .map_err(|error| format!("assets3d_manifest_read: {}: {error}", path.display()))?;
    if metadata.len() > MAX_MANIFEST_BYTES {
        return Err("assets3d_manifest_too_large".into());
    }
    let bytes = fs::read(path)
        .map_err(|error| format!("assets3d_manifest_read: {}: {error}", path.display()))?;
    let manifest: Asset3dManifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("assets3d_manifest_invalid: {error}"))?;
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!("assets3d_manifest_version: attendu {MANIFEST_SCHEMA}").into());
    }
    if manifest.assets.len() > MAX_ASSETS {
        return Err("assets3d_count_quota".into());
    }
    let root = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .map_err(|error| format!("assets3d_root_invalid: {error}"))?;
    let mut entries = BTreeMap::new();
    let mut total = 0_u64;
    for entry in manifest.assets {
        validate_id(&entry.id)?;
        if entry.size > MAX_ASSET_BYTES {
            return Err(format!("assets3d_asset_quota: {}", entry.id).into());
        }
        total = total
            .checked_add(entry.size)
            .ok_or("assets3d_bytes_quota")?;
        if total > MAX_TOTAL_BYTES {
            return Err("assets3d_bytes_quota".into());
        }
        if entries.insert(entry.id.clone(), entry).is_some() {
            return Err("assets3d_duplicate_id".into());
        }
    }

    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        for (id, entry) in entries {
            let sender = sender.clone();
            let root = root.clone();
            scope.spawn(move || {
                let result = load_entry(&root, &entry, cache_root);
                let _ = sender.send((id, result));
            });
        }
    });
    drop(sender);
    let mut loaded = BTreeMap::new();
    for (id, result) in receiver {
        loaded.insert(id, result?);
    }
    Ok(loaded)
}

fn load_entry(root: &Path, entry: &Asset3dEntry, cache_root: Option<&Path>) -> Result<Asset3d> {
    let path = confined(root, Path::new(&entry.path))?;
    let source_bytes =
        fs::read(&path).map_err(|error| format!("assets3d_read: {}: {error}", path.display()))?;
    if source_bytes.len() as u64 != entry.size {
        return Err(format!("assets3d_size_mismatch: {}", entry.id).into());
    }
    if checksum_bytes(&source_bytes) != entry.checksum {
        return Err(format!("assets3d_checksum_mismatch: {}", entry.id).into());
    }
    let bytes = cache_root
        .and_then(|root| read_cache(root, entry))
        .unwrap_or_else(|| {
            if let Some(root) = cache_root {
                write_cache(root, entry, &source_bytes);
            }
            source_bytes
        });
    match entry.kind {
        Asset3dType::Mesh => {
            let document: MeshDocument = serde_json::from_slice(&bytes)
                .map_err(|error| format!("mesh3d_invalid: {error}"))?;
            if document.schema != MESH_SCHEMA {
                return Err(format!("mesh3d_version: attendu {MESH_SCHEMA}").into());
            }
            if document.mesh.id != entry.id {
                return Err(format!("assets3d_id_mismatch: {}", entry.id).into());
            }
            Ok(Asset3d::Mesh(document.mesh))
        }
        Asset3dType::Material => {
            let document: MaterialDocument = serde_json::from_slice(&bytes)
                .map_err(|error| format!("material3d_invalid: {error}"))?;
            if document.schema != MATERIAL_SCHEMA {
                return Err(format!("material3d_version: attendu {MATERIAL_SCHEMA}").into());
            }
            if document.material.id != entry.id {
                return Err(format!("assets3d_id_mismatch: {}", entry.id).into());
            }
            Ok(Asset3d::Material(document.material))
        }
        Asset3dType::Texture => {
            if bytes.len() as u64 > MAX_TEXTURE_BYTES
                || bytes.is_empty()
                || !is_supported_texture(&bytes)
            {
                return Err(format!("texture3d_invalid: {}", entry.id).into());
            }
            Ok(Asset3d::Texture(Texture3d {
                id: entry.id.clone(),
                bytes,
            }))
        }
    }
}

pub fn import(input: &Path, kind: Asset3dType, output: &Path) -> Result<()> {
    let metadata = fs::metadata(input)
        .map_err(|error| format!("asset3d_import_read: {}: {error}", input.display()))?;
    if metadata.len() > MAX_ASSET_BYTES {
        return Err("asset3d_import_too_large".into());
    }
    let bytes = fs::read(input)
        .map_err(|error| format!("asset3d_import_read: {}: {error}", input.display()))?;
    let mut canonical = match kind {
        Asset3dType::Mesh => {
            let document: MeshDocument = serde_json::from_slice(&bytes)
                .map_err(|error| format!("mesh3d_invalid: {error}"))?;
            if document.schema != MESH_SCHEMA {
                return Err(format!("mesh3d_version: attendu {MESH_SCHEMA}").into());
            }
            validate_id(&document.mesh.id)?;
            serde_json::to_vec_pretty(&document)
        }
        Asset3dType::Material => {
            let document: MaterialDocument = serde_json::from_slice(&bytes)
                .map_err(|error| format!("material3d_invalid: {error}"))?;
            if document.schema != MATERIAL_SCHEMA {
                return Err(format!("material3d_version: attendu {MATERIAL_SCHEMA}").into());
            }
            validate_id(&document.material.id)?;
            serde_json::to_vec_pretty(&document)
        }
        Asset3dType::Texture => {
            return Err("asset3d_import_texture_not_supported: copiez la texture et mettez a jour le manifeste".into());
        }
    }
    .map_err(|error| format!("asset3d_import_serialize: {error}"))?;
    canonical.push(b'\n');
    publish_atomic(output, &canonical)
}

fn publish_atomic(output: &Path, bytes: &[u8]) -> Result<()> {
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("asset3d_import_output: {}: {error}", parent.display()))?;
    if output.exists() {
        return Err("asset3d_import_output_exists".into());
    }
    let temporary = parent.join(format!(".aetherion-asset3d-import-{}", std::process::id()));
    if temporary.exists() {
        fs::remove_file(&temporary).map_err(|error| format!("asset3d_import_cleanup: {error}"))?;
    }
    fs::write(&temporary, bytes).map_err(|error| format!("asset3d_import_write: {error}"))?;
    if let Err(error) = fs::rename(&temporary, output) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("asset3d_import_publish: {error}").into());
    }
    Ok(())
}

fn confined(root: &Path, requested: &Path) -> Result<PathBuf> {
    if requested.is_absolute()
        || requested.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err("assets3d_path_traversal".into());
    }
    let candidate = root.join(requested);
    let canonical = candidate
        .canonicalize()
        .map_err(|error| format!("assets3d_path_invalid: {}: {error}", candidate.display()))?;
    if !canonical.starts_with(root) {
        return Err("assets3d_path_traversal".into());
    }
    Ok(canonical)
}

fn cache_path(cache_root: &Path, entry: &Asset3dEntry) -> PathBuf {
    cache_root.join(format!(
        "aetherion-asset-{}-{}-{}-{:016x}.bin",
        asset_kind_name(entry.kind),
        CACHE_FORMAT_VERSION,
        IMPORTER_VERSION,
        entry.checksum
    ))
}

fn asset_kind_name(kind: Asset3dType) -> &'static str {
    match kind {
        Asset3dType::Mesh => "mesh",
        Asset3dType::Material => "material",
        Asset3dType::Texture => "texture",
    }
}

fn read_cache(cache_root: &Path, entry: &Asset3dEntry) -> Option<Vec<u8>> {
    let bytes = fs::read(cache_path(cache_root, entry)).ok()?;
    (bytes.len() as u64 == entry.size && checksum_bytes(&bytes) == entry.checksum).then_some(bytes)
}

fn write_cache(cache_root: &Path, entry: &Asset3dEntry, bytes: &[u8]) {
    if fs::create_dir_all(cache_root).is_err() {
        return;
    }
    let target = cache_path(cache_root, entry);
    if target.exists() {
        return;
    }
    let temporary = cache_root.join(format!(
        ".aetherion-cache-{}-{}",
        entry.checksum,
        std::process::id()
    ));
    if fs::write(&temporary, bytes).is_ok() && fs::rename(&temporary, &target).is_err() {
        let _ = fs::remove_file(&temporary);
    }
}

fn is_supported_texture(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\x89PNG\r\n\x1a\n") || bytes.starts_with(&[0xff, 0xd8, 0xff])
}

fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(format!("assets3d_id_invalid: {id}").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "aetherion-assets3d-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn traversal_is_rejected() {
        let root = std::env::current_dir().unwrap().canonicalize().unwrap();
        assert!(confined(&root, Path::new("../mesh.json")).is_err());
    }

    #[test]
    fn binary_texture_assets_are_checksum_checked_and_loaded() {
        let directory = temporary_directory("texture");
        fs::create_dir_all(&directory).unwrap();
        let texture = b"\x89PNG\r\n\x1a\ntexture";
        fs::write(directory.join("albedo.png"), texture).unwrap();
        let manifest = serde_json::json!({
            "schema": MANIFEST_SCHEMA,
            "assets": [{
                "id": "albedo",
                "path": "albedo.png",
                "type": "texture",
                "size": texture.len(),
                "checksum": checksum_bytes(texture)
            }]
        });
        let manifest_path = directory.join("assets.json");
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let loaded = load_manifest(&manifest_path).unwrap();
        assert!(matches!(loaded["albedo"], Asset3d::Texture(_)));
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn concurrent_loading_is_canonical_and_reloads_changed_content() {
        let directory = temporary_directory("concurrent");
        fs::create_dir_all(&directory).unwrap();
        let mesh = br#"{"schema":"aetherion.mesh3d/v1","mesh":{"id":"mesh","vertices":[{"x":0,"y":0,"z":1},{"x":1,"y":0,"z":1},{"x":0,"y":1,"z":1}],"triangles":[[0,1,2]]}}"#;
        let material = br#"{"schema":"aetherion.material3d/v1","material":{"id":"material","color":[1,2,3],"opacity":1000}}"#;
        fs::write(directory.join("mesh.json"), mesh).unwrap();
        fs::write(directory.join("material.json"), material).unwrap();
        let manifest = serde_json::json!({
            "schema": MANIFEST_SCHEMA,
            "assets": [
                {"id":"material","path":"material.json","type":"material","size":material.len(),"checksum":checksum_bytes(material)},
                {"id":"mesh","path":"mesh.json","type":"mesh","size":mesh.len(),"checksum":checksum_bytes(mesh)}
            ]
        });
        let manifest_path = directory.join("assets.json");
        fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

        for _ in 0..2 {
            let loaded = load_manifest(&manifest_path).unwrap();
            assert_eq!(
                loaded.keys().cloned().collect::<Vec<_>>(),
                ["material", "mesh"]
            );
            assert!(matches!(loaded["mesh"], Asset3d::Mesh(_)));
            assert!(matches!(loaded["material"], Asset3d::Material(_)));
        }

        let cache = directory.join("cache");
        let cached = load_manifest_cached(&manifest_path, &cache).unwrap();
        assert!(matches!(cached["mesh"], Asset3d::Mesh(_)));
        assert!(matches!(cached["material"], Asset3d::Material(_)));
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 2);
        let cached_again = load_manifest_cached(&manifest_path, &cache).unwrap();
        assert_eq!(
            cached_again.keys().collect::<Vec<_>>(),
            cached.keys().collect::<Vec<_>>()
        );
        fs::write(
            fs::read_dir(&cache)
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path(),
            b"corrupt",
        )
        .unwrap();
        load_manifest_cached(&manifest_path, &cache).unwrap();

        fs::write(directory.join("mesh.json"), b"changed").unwrap();
        assert!(
            load_manifest(&manifest_path)
                .unwrap_err()
                .to_string()
                .contains("size_mismatch")
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
