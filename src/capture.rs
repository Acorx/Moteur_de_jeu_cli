use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::project::{CameraConfig, RenderConfig};
use crate::render::{self, CameraManifest, Dimensions, SegmentationEntry, VisibleEntity};
use crate::simulation::World;

pub const MAX_VIEWS: usize = 64;
pub const MAX_PIXELS_PER_VIEW: u64 = 16_777_216;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    #[default]
    Ppm,
    Png,
}

impl ImageFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Ppm => "ppm",
            Self::Png => "png",
        }
    }
    pub fn media_name(self) -> &'static str {
        match self {
            Self::Ppm => "ppm-p6",
            Self::Png => "png",
        }
    }
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "ppm" => Ok(Self::Ppm),
            "png" => Ok(Self::Png),
            _ => Err("--format doit valoir ppm ou png".into()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Color,
    Depth,
    Normals,
    Segmentation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Channels(BTreeSet<Channel>);

impl Default for Channels {
    fn default() -> Self {
        Self(BTreeSet::from([Channel::Color]))
    }
}
impl Channels {
    pub fn parse(value: &str) -> Result<Self> {
        if value.is_empty() {
            return Err("--channels ne peut pas être vide".into());
        }
        let mut set = BTreeSet::new();
        for item in value.split(',') {
            let channel = match item {
                "color" => Channel::Color,
                "depth" => Channel::Depth,
                "normals" => Channel::Normals,
                "segmentation" => Channel::Segmentation,
                "" => return Err("canal vide dans --channels".into()),
                other => return Err(format!("canal inconnu: {other}").into()),
            };
            if !set.insert(channel) {
                return Err(format!("canal dupliqué: {item}").into());
            }
        }
        if !set.contains(&Channel::Color) {
            return Err("--channels doit inclure color".into());
        }
        Ok(Self(set))
    }
    pub fn contains(&self, channel: Channel) -> bool {
        self.0.contains(&channel)
    }
    pub fn is_default(&self) -> bool {
        self.0.len() == 1 && self.contains(Channel::Color)
    }
}

#[derive(Debug, Serialize)]
struct ChannelManifest {
    name: &'static str,
    file: String,
    encoding: &'static str,
    checksum: u64,
    dimensions: Dimensions,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ViewsFile {
    pub schema: String,
    pub views: Vec<ViewSpec>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ViewSpec {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub camera: CameraConfig,
    #[serde(default)]
    pub format: ImageFormat,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<String>,
}

#[derive(Debug, Serialize)]
struct SingleManifest<'a> {
    schema: &'static str,
    project: &'a str,
    tick: u64,
    world_checksum: u64,
    image_checksum: u64,
    path: String,
    format: &'static str,
    dimensions: Dimensions,
    camera: CameraManifest,
    visible_entities: &'a [VisibleEntity],
    #[serde(skip_serializing_if = "Option::is_none")]
    channels: Option<Vec<ChannelManifest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    segmentation_mapping: Option<&'a [SegmentationEntry]>,
}

#[derive(Debug, Serialize)]
struct MultiManifest<'a> {
    schema: &'static str,
    project: &'a str,
    tick: u64,
    world_checksum: u64,
    views: Vec<MultiViewManifest>,
}

#[derive(Debug, Serialize)]
struct MultiViewManifest {
    name: String,
    image_checksum: u64,
    path: String,
    format: &'static str,
    dimensions: Dimensions,
    camera: CameraManifest,
    visible_entities: Vec<VisibleEntity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channels: Option<Vec<ChannelManifest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    segmentation_mapping: Option<Vec<SegmentationEntry>>,
}

pub fn load_views(path: &Path) -> Result<ViewsFile> {
    let bytes = fs::read(path).map_err(|e| format!("lecture de {}: {e}", path.display()))?;
    if bytes.len() > 1_048_576 {
        return Err("fichier de vues limité à 1 MiB".into());
    }
    let views: ViewsFile = serde_json::from_slice(&bytes)
        .map_err(|e| format!("vues invalides dans {}: {e}", path.display()))?;
    validate_views(&views)?;
    Ok(views)
}

pub fn validate_views(value: &ViewsFile) -> Result<()> {
    if value.schema != "aetherion.capture-views/v1" {
        return Err("schéma de vues attendu: aetherion.capture-views/v1".into());
    }
    if value.views.is_empty() || value.views.len() > MAX_VIEWS {
        return Err(format!("le lot doit contenir 1 à {MAX_VIEWS} vues").into());
    }
    let mut names = std::collections::BTreeSet::new();
    for view in &value.views {
        if view.name.is_empty()
            || view.name.len() > 64
            || !view
                .name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        {
            return Err(format!("nom de vue invalide: {}", view.name).into());
        }
        if !names.insert(view.name.to_ascii_lowercase()) {
            return Err(format!("nom de vue dupliqué: {}", view.name).into());
        }
        let pixels = u64::from(view.width)
            .checked_mul(u64::from(view.height))
            .ok_or("dimensions hors limite")?;
        if view.width == 0 || view.height == 0 || pixels > MAX_PIXELS_PER_VIEW {
            return Err(format!("dimensions invalides pour {}", view.name).into());
        }
        if let Some(channels) = &view.channels {
            Channels::parse(channels)?;
        }
        if !(1..=1024).contains(&view.camera.pixels_per_unit) {
            return Err(format!("zoom invalide pour {}", view.name).into());
        }
    }
    Ok(())
}

pub fn capture(
    world: &World,
    config: &RenderConfig,
    output: &Path,
    format: ImageFormat,
    textures: &BTreeMap<String, crate::assets::Texture>,
    channels: &Channels,
) -> Result<PathBuf> {
    let (buffers, visible) = render::render_buffers_with_textures(world, config, textures)?;
    let image = &buffers.color;
    let bytes = encode(image, format)?;
    let auxiliary = auxiliary_files(output, &buffers, channels)?;
    let channel_manifest = if channels.is_default() {
        None
    } else {
        Some(
            auxiliary
                .iter()
                .map(|file| ChannelManifest {
                    name: file.name,
                    file: normalized(&file.path),
                    encoding: file.encoding,
                    checksum: render::checksum_bytes(&file.bytes),
                    dimensions: Dimensions {
                        width: image.width,
                        height: image.height,
                    },
                })
                .collect(),
        )
    };
    let manifest = SingleManifest {
        schema: "aetherion.capture/v1",
        project: &world.project,
        tick: world.tick,
        world_checksum: world.checksum(),
        image_checksum: render::checksum_bytes(&bytes),
        path: normalized(output),
        format: format.media_name(),
        dimensions: Dimensions {
            width: image.width,
            height: image.height,
        },
        camera: camera_manifest(&config.camera),
        visible_entities: &visible,
        channels: channel_manifest,
        segmentation_mapping: channels
            .contains(Channel::Segmentation)
            .then_some(buffers.segmentation_mapping.as_slice()),
    };
    let manifest_bytes = json_bytes(&manifest)?;
    write_set_atomic(
        output,
        &bytes,
        &render::manifest_path(output),
        &manifest_bytes,
        &auxiliary,
    )?;
    Ok(render::manifest_path(output))
}

pub fn capture_multi(
    world: &World,
    base: &RenderConfig,
    views: &ViewsFile,
    output_dir: &Path,
    textures: &std::collections::BTreeMap<String, crate::assets::Texture>,
    channels: &Channels,
) -> Result<PathBuf> {
    validate_views(views)?;
    if output_dir.exists() {
        return Err(format!("le dossier cible existe déjà: {}", output_dir.display()).into());
    }
    let parent = output_dir
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| format!("création de {}: {e}", parent.display()))?;
    let name = output_dir
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or("nom de dossier cible invalide")?;
    let stage = parent.join(format!(".{name}.aetherion-stage-{}", std::process::id()));
    if stage.exists() {
        fs::remove_dir_all(&stage).map_err(|e| format!("nettoyage staging: {e}"))?;
    }
    fs::create_dir(&stage).map_err(|e| format!("création staging: {e}"))?;

    let result = (|| -> Result<PathBuf> {
        let mut entries = Vec::with_capacity(views.views.len());
        for view in &views.views {
            let config = RenderConfig {
                width: view.width,
                height: view.height,
                background: base.background,
                camera: view.camera.clone(),
            };
            let selected = view
                .channels
                .as_deref()
                .map(Channels::parse)
                .transpose()?
                .unwrap_or_else(|| channels.clone());
            let (buffers, visible_entities) =
                render::render_buffers_with_textures(world, &config, textures)?;
            let image = &buffers.color;
            let bytes = encode(image, view.format)?;
            let file_name = format!("{}.{}", view.name, view.format.extension());
            fs::write(stage.join(&file_name), &bytes)
                .map_err(|e| format!("écriture vue {file_name}: {e}"))?;
            let aux = auxiliary_files(&PathBuf::from(&file_name), &buffers, &selected)?;
            for file in &aux {
                fs::write(stage.join(file.path.file_name().unwrap()), &file.bytes)
                    .map_err(|e| format!("écriture canal: {e}"))?;
            }
            let channel_manifest = if selected.is_default() {
                None
            } else {
                Some(
                    aux.iter()
                        .map(|file| ChannelManifest {
                            name: file.name,
                            file: file
                                .path
                                .file_name()
                                .unwrap()
                                .to_string_lossy()
                                .into_owned(),
                            encoding: file.encoding,
                            checksum: render::checksum_bytes(&file.bytes),
                            dimensions: Dimensions {
                                width: image.width,
                                height: image.height,
                            },
                        })
                        .collect(),
                )
            };
            entries.push(MultiViewManifest {
                name: view.name.clone(),
                image_checksum: render::checksum_bytes(&bytes),
                path: file_name,
                format: view.format.media_name(),
                dimensions: Dimensions {
                    width: view.width,
                    height: view.height,
                },
                camera: camera_manifest(&view.camera),
                visible_entities,
                channels: channel_manifest,
                segmentation_mapping: selected
                    .contains(Channel::Segmentation)
                    .then_some(buffers.segmentation_mapping),
            });
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        let manifest = MultiManifest {
            schema: "aetherion.capture-multi/v1",
            project: &world.project,
            tick: world.tick,
            world_checksum: world.checksum(),
            views: entries,
        };
        fs::write(stage.join("manifest.json"), json_bytes(&manifest)?)
            .map_err(|e| format!("écriture manifeste agrégé: {e}"))?;
        fs::rename(&stage, output_dir)
            .map_err(|e| format!("publication atomique de {}: {e}", output_dir.display()))?;
        Ok(output_dir.join("manifest.json"))
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    result
}

fn camera_manifest(camera: &CameraConfig) -> CameraManifest {
    CameraManifest {
        x: camera.x,
        y: camera.y,
        pixels_per_unit: camera.pixels_per_unit,
    }
}
fn normalized(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|e| format!("sérialisation manifeste: {e}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}
fn encode(image: &render::Image, format: ImageFormat) -> Result<Vec<u8>> {
    match format {
        ImageFormat::Ppm => Ok(render::encode_ppm(image)),
        ImageFormat::Png => encode_png(image),
    }
}

struct AuxiliaryFile {
    name: &'static str,
    path: PathBuf,
    encoding: &'static str,
    bytes: Vec<u8>,
}
fn auxiliary_path(output: &Path, suffix: &str) -> PathBuf {
    let stem = output
        .file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or("capture");
    output.with_file_name(format!("{stem}.{suffix}"))
}
fn auxiliary_files(
    output: &Path,
    buffers: &render::RenderBuffers,
    channels: &Channels,
) -> Result<Vec<AuxiliaryFile>> {
    let mut files = Vec::new();
    if channels.contains(Channel::Depth) {
        files.push(AuxiliaryFile {
            name: "depth",
            path: auxiliary_path(output, "depth.pgm"),
            encoding: "pgm-p5-u16be",
            bytes: encode_depth_pgm(buffers),
        });
    }
    if channels.contains(Channel::Normals) {
        files.push(AuxiliaryFile {
            name: "normals",
            path: auxiliary_path(output, "normals.png"),
            encoding: "png-rgb8",
            bytes: encode_png(&buffers.normals)?,
        });
    }
    if channels.contains(Channel::Segmentation) {
        let mut pixels = Vec::with_capacity(buffers.segmentation.len() * 3);
        for value in &buffers.segmentation {
            pixels.extend_from_slice(&value.to_be_bytes()[1..]);
        }
        files.push(AuxiliaryFile {
            name: "segmentation",
            path: auxiliary_path(output, "segmentation.png"),
            encoding: "png-rgb24-id",
            bytes: encode_png(&render::Image {
                width: buffers.color.width,
                height: buffers.color.height,
                pixels,
            })?,
        });
    }
    Ok(files)
}
pub fn encode_depth_pgm(buffers: &render::RenderBuffers) -> Vec<u8> {
    let mut bytes = format!(
        "P5\n{} {}\n65535\n",
        buffers.color.width, buffers.color.height
    )
    .into_bytes();
    for value in &buffers.depth {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes
}
fn write_set_atomic(
    image_path: &Path,
    image: &[u8],
    manifest_path: &Path,
    manifest: &[u8],
    auxiliary: &[AuxiliaryFile],
) -> Result<()> {
    let parent = image_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| format!("création de {}: {e}", parent.display()))?;
    let mut targets = vec![image_path.to_path_buf(), manifest_path.to_path_buf()];
    targets.extend(auxiliary.iter().map(|file| file.path.clone()));
    if targets.iter().any(|path| path.exists()) {
        return Err("la capture cible existe déjà".into());
    }
    let stage = parent.join(format!(".aetherion-capture-stage-{}", std::process::id()));
    if stage.exists() {
        fs::remove_dir_all(&stage).map_err(|e| format!("nettoyage staging: {e}"))?;
    }
    fs::create_dir(&stage).map_err(|e| format!("création staging: {e}"))?;
    let result = (|| -> Result<()> {
        let mut entries: Vec<(&Path, &[u8])> = vec![(image_path, image), (manifest_path, manifest)];
        entries.extend(
            auxiliary
                .iter()
                .map(|file| (file.path.as_path(), file.bytes.as_slice())),
        );
        for (index, (_, bytes)) in entries.iter().enumerate() {
            fs::write(stage.join(index.to_string()), bytes)
                .map_err(|e| format!("écriture temporaire: {e}"))?;
        }
        let mut published = Vec::new();
        for (index, (target, _)) in entries.iter().enumerate() {
            if let Err(error) = fs::rename(stage.join(index.to_string()), target) {
                for path in published {
                    let _ = fs::remove_file(path);
                }
                return Err(format!("publication atomique: {error}").into());
            }
            published.push(target.to_path_buf());
        }
        Ok(())
    })();
    let _ = fs::remove_dir_all(&stage);
    result
}

pub fn encode_png(image: &render::Image) -> Result<Vec<u8>> {
    let expected = (image.width as usize)
        .checked_mul(image.height as usize)
        .and_then(|n| n.checked_mul(3))
        .ok_or("image trop grande")?;
    if image.pixels.len() != expected {
        return Err("taille du tampon RGB incohérente".into());
    }
    let stride = image.width as usize * 3;
    let mut raw = Vec::with_capacity(expected + image.height as usize);
    for row in image.pixels.chunks_exact(stride) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut zlib = vec![0x78, 0x01];
    let mut remaining = raw.as_slice();
    while !remaining.is_empty() {
        let count = remaining.len().min(65_535);
        let final_block = count == remaining.len();
        zlib.push(u8::from(final_block));
        zlib.extend_from_slice(&(count as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(count as u16)).to_le_bytes());
        zlib.extend_from_slice(&remaining[..count]);
        remaining = &remaining[count..];
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&image.width.to_be_bytes());
    ihdr.extend_from_slice(&image.height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &zlib);
    chunk(&mut png, b"IEND", &[]);
    Ok(png)
}
fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc_data = Vec::with_capacity(4 + data.len());
    crc_data.extend_from_slice(kind);
    crc_data.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_data).to_be_bytes());
}
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & (0u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}
fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in bytes {
        a = (a + u32::from(byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn png_is_stable_and_has_expected_chunks() {
        let image = render::Image {
            width: 2,
            height: 1,
            pixels: vec![255, 0, 0, 0, 255, 0],
        };
        let a = encode_png(&image).unwrap();
        let b = encode_png(&image).unwrap();
        assert_eq!(a, b);
        assert!(a.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert_eq!(&a[12..16], b"IHDR");
        assert!(a.windows(4).any(|v| v == b"IDAT"));
        assert!(a.windows(4).any(|v| v == b"IEND"));
    }
    #[test]
    fn views_reject_collisions_and_bad_zoom() {
        let mut value = ViewsFile {
            schema: "aetherion.capture-views/v1".into(),
            views: vec![ViewSpec {
                name: "main".into(),
                width: 1,
                height: 1,
                camera: CameraConfig::default(),
                format: ImageFormat::Png,
                channels: None,
            }],
        };
        assert!(validate_views(&value).is_ok());
        value.views.push(value.views[0].clone());
        assert!(validate_views(&value).is_err());
        value.views.pop();
        value.views[0].camera.pixels_per_unit = 0;
        assert!(validate_views(&value).is_err());
    }
}
