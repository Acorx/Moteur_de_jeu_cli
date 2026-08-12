use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::Result;
use crate::render::checksum_bytes;

pub const BUNDLE_SCHEMA: &str = "aetherion.bundle/v1";

#[derive(Serialize)]
struct BundleManifest {
    schema: &'static str,
    files: Vec<BundleFile>,
}
#[derive(Serialize)]
struct BundleFile {
    path: String,
    checksum_fnv1a: u64,
    size: u64,
}
#[derive(Serialize)]
pub struct InspectReport {
    pub schema: &'static str,
    pub entries: Vec<ZipEntry>,
}
#[derive(Serialize)]
pub struct ZipEntry {
    pub path: String,
    pub size: u32,
    pub checksum_fnv1a: u64,
}
struct RawEntry {
    name: String,
    data: Vec<u8>,
    crc: u32,
    offset: u32,
}

pub fn create(path: &Path, output: &Path) -> Result<()> {
    let root =
        fs::canonicalize(path).map_err(|e| format!("bundle_root_read: {}: {e}", path.display()))?;
    let output = absolute_path(output)?;
    let mut sources = Vec::new();
    collect(&root, &root, &output, &mut sources)?;
    sources.sort_by(|a, b| a.0.cmp(&b.0));
    let files = sources
        .iter()
        .map(|(name, data)| BundleFile {
            path: name.clone(),
            checksum_fnv1a: checksum_bytes(data),
            size: data.len() as u64,
        })
        .collect();
    let manifest = serde_json::to_vec_pretty(&BundleManifest {
        schema: BUNDLE_SCHEMA,
        files,
    })
    .map_err(|e| format!("bundle_manifest_serialize: {e}"))?;
    let mut entries = vec![("aetherion.bundle.json".to_owned(), manifest)];
    entries.extend(sources);
    write_zip(&output, entries)
}

pub fn inspect(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|e| format!("bundle_read: {}: {e}", path.display()))?;
    let entries = read_entries(&bytes)?;
    serde_json::to_string_pretty(&InspectReport {
        schema: "aetherion.bundle-inspect/v1",
        entries,
    })
    .map_err(|e| format!("bundle_inspect_serialize: {e}").into())
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("bundle_current_dir: {e}"))?
            .join(path)
    };
    if path.exists() {
        return fs::canonicalize(&path)
            .map_err(|e| format!("bundle_output_read: {}: {e}", path.display()).into());
    }
    let parent = path.parent().ok_or("bundle_output_parent")?;
    let name = path.file_name().ok_or("bundle_output_name")?;
    fs::canonicalize(parent)
        .map_err(|e| format!("bundle_output_parent: {}: {e}", parent.display()))
        .map(|parent| parent.join(name))
        .map_err(Into::into)
}

fn collect(
    root: &Path,
    directory: &Path,
    output: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<()> {
    for entry in fs::read_dir(directory)
        .map_err(|e| format!("bundle_read_dir: {}: {e}", directory.display()))?
    {
        let entry = entry.map_err(|e| format!("bundle_entry: {e}"))?;
        let path = entry.path();
        let kind = entry
            .file_type()
            .map_err(|e| format!("bundle_entry_type: {e}"))?;
        if kind.is_symlink() {
            return Err("bundle_symlink_not_allowed".into());
        }
        if kind.is_dir() {
            collect(root, &path, output, files)?;
        } else if kind.is_file() {
            if path == output {
                continue;
            }
            let name = path
                .strip_prefix(root)
                .map_err(|_| "bundle_relative_path")?
                .to_string_lossy()
                .replace('\\', "/");
            let data =
                fs::read(&path).map_err(|e| format!("bundle_read: {}: {e}", path.display()))?;
            files.push((name, data));
        }
    }
    Ok(())
}
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
fn le16(out: &mut Vec<u8>, value: u16) {
    out.extend(value.to_le_bytes());
}
fn le32(out: &mut Vec<u8>, value: u32) {
    out.extend(value.to_le_bytes());
}
fn write_zip(output: &Path, items: Vec<(String, Vec<u8>)>) -> Result<()> {
    let mut out = Vec::new();
    let mut entries = Vec::new();
    for (name, data) in items {
        let name_bytes = name.as_bytes();
        let offset = out.len() as u32;
        let crc = crc32(&data);
        le32(&mut out, 0x0403_4b50);
        le16(&mut out, 20);
        le16(&mut out, 0);
        le16(&mut out, 0);
        le16(&mut out, 0);
        le16(&mut out, 0);
        le32(&mut out, crc);
        le32(&mut out, data.len() as u32);
        le32(&mut out, data.len() as u32);
        le16(&mut out, name_bytes.len() as u16);
        le16(&mut out, 0);
        out.extend(name_bytes);
        out.extend(&data);
        entries.push(RawEntry {
            name,
            data,
            crc,
            offset,
        });
    }
    let central = out.len() as u32;
    for entry in &entries {
        let name = entry.name.as_bytes();
        le32(&mut out, 0x0201_4b50);
        le16(&mut out, 20);
        le16(&mut out, 20);
        le16(&mut out, 0);
        le16(&mut out, 0);
        le16(&mut out, 0);
        le16(&mut out, 0);
        le32(&mut out, entry.crc);
        le32(&mut out, entry.data.len() as u32);
        le32(&mut out, entry.data.len() as u32);
        le16(&mut out, name.len() as u16);
        le16(&mut out, 0);
        le16(&mut out, 0);
        le16(&mut out, 0);
        le16(&mut out, 0);
        le32(&mut out, 0);
        le32(&mut out, entry.offset);
        out.extend(name);
    }
    let size = out.len() as u32 - central;
    le32(&mut out, 0x0605_4b50);
    le16(&mut out, 0);
    le16(&mut out, 0);
    le16(&mut out, entries.len() as u16);
    le16(&mut out, entries.len() as u16);
    le32(&mut out, size);
    le32(&mut out, central);
    le16(&mut out, 0);
    atomic_write(output, &out)
}
fn read_u16(bytes: &[u8], at: usize) -> Result<u16> {
    bytes
        .get(at..at + 2)
        .and_then(|v| v.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| "bundle_zip_invalid".into())
}
fn read_u32(bytes: &[u8], at: usize) -> Result<u32> {
    bytes
        .get(at..at + 4)
        .and_then(|v| v.try_into().ok())
        .map(u32::from_le_bytes)
        .ok_or_else(|| "bundle_zip_invalid".into())
}
fn read_entries(bytes: &[u8]) -> Result<Vec<ZipEntry>> {
    let end = bytes
        .windows(4)
        .rposition(|x| x == [0x50, 0x4b, 0x05, 0x06])
        .ok_or("bundle_zip_eocd_missing")?;
    let count = read_u16(bytes, end + 10)? as usize;
    let mut at = read_u32(bytes, end + 16)? as usize;
    let mut result = Vec::new();
    for _ in 0..count {
        if read_u32(bytes, at)? != 0x0201_4b50 {
            return Err("bundle_zip_central_invalid".into());
        }
        let size = read_u32(bytes, at + 24)?;
        let name_len = read_u16(bytes, at + 28)? as usize;
        let extra = read_u16(bytes, at + 30)? as usize;
        let comment = read_u16(bytes, at + 32)? as usize;
        let offset = read_u32(bytes, at + 42)? as usize;
        let name = std::str::from_utf8(
            bytes
                .get(at + 46..at + 46 + name_len)
                .ok_or("bundle_zip_invalid")?,
        )
        .map_err(|_| "bundle_zip_name")?
        .to_owned();
        let local_name = read_u16(bytes, offset + 26)? as usize;
        let local_extra = read_u16(bytes, offset + 28)? as usize;
        let data_at = offset + 30 + local_name + local_extra;
        let data = bytes
            .get(data_at..data_at + size as usize)
            .ok_or("bundle_zip_invalid")?;
        result.push(ZipEntry {
            path: name,
            size,
            checksum_fnv1a: checksum_bytes(data),
        });
        at += 46 + name_len + extra + comment;
    }
    Ok(result)
}
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|e| format!("bundle_mkdir: {}: {e}", parent.display()))?;
    }
    let mut tmp = PathBuf::from(path);
    tmp.set_extension(format!("tmp-{}", std::process::id()));
    fs::write(&tmp, bytes).map_err(|e| format!("bundle_write: {}: {e}", tmp.display()))?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("bundle_replace: {}: {e}", path.display()))?;
    }
    fs::rename(&tmp, path).map_err(|e| format!("bundle_rename: {}: {e}", path.display()).into())
}
