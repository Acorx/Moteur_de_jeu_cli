use serde::{Deserialize, Serialize};

use crate::Result;
use crate::capture::Channels;
use crate::render::{Image, RenderBuffers};

pub const SCENE_SCHEMA: &str = "aetherion.scene3d/v1";
pub const CAPTURE_SCHEMA: &str = "aetherion.capture3d/v1";
pub const MAX_SCENE_BYTES: u64 = 1_048_576;
pub const MAX_TRIANGLES: usize = 100_000;
pub const MAX_VERTICES: usize = MAX_TRIANGLES * 3;
pub const MAX_PIXELS: u64 = 16_777_216;
pub const MAX_MESHES: usize = 10_000;
pub const MAX_MATERIALS: usize = 10_000;
pub const MAX_OBJECTS: usize = 100_000;
pub const MAX_ANIMATIONS: usize = 10_000;
pub const MAX_TRACKS: usize = 100_000;
pub const MAX_KEYFRAMES: usize = 1_000_000;
/// Quantization used for imported unit normals in the versioned scene format.
pub const NORMAL_SCALE: i32 = 1_000_000;
/// Quantization used for imported texture coordinates in the versioned scene format.
pub const UV_SCALE: i32 = 1_000_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scene3d {
    pub schema: String,
    #[serde(default)]
    pub camera: Camera3d,
    #[serde(default)]
    pub background: [u8; 3],
    #[serde(default)]
    pub triangles: Vec<Triangle>,
    #[serde(default)]
    pub meshes: Vec<Mesh3d>,
    #[serde(default)]
    pub materials: Vec<Material3d>,
    #[serde(default)]
    pub objects: Vec<Object3d>,
    #[serde(default)]
    pub animations: Vec<Animation3d>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Camera3d {
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub z: i32,
    #[serde(default = "default_scale")]
    pub pixels_per_unit: u32,
}

impl Default for Camera3d {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            pixels_per_unit: default_scale(),
        }
    }
}

const fn default_scale() -> u32 {
    16
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Vertex3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Triangle {
    pub id: u32,
    pub vertices: [Vertex3; 3],
    pub color: [u8; 3],
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Mesh3d {
    pub id: String,
    #[serde(default)]
    pub vertices: Vec<Vertex3>,
    #[serde(default)]
    pub triangles: Vec<[u32; 3]>,
    /// Optional per-vertex normals quantized with [`NORMAL_SCALE`].
    /// An empty vector keeps compatibility with legacy flat-shaded meshes.
    #[serde(default)]
    pub normals: Vec<[i32; 3]>,
    /// Optional per-vertex UV coordinates quantized with [`UV_SCALE`].
    /// An empty vector means that the mesh has no texture coordinates.
    #[serde(default)]
    pub uvs: Vec<[i32; 2]>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Material3d {
    pub id: String,
    #[serde(default = "default_material_color")]
    pub color: [u8; 3],
    #[serde(default = "default_opacity")]
    pub opacity: u16,
    /// External texture asset ID. The CPU renderer intentionally ignores it.
    #[serde(default)]
    pub base_color_texture: Option<String>,
}

impl Default for Material3d {
    fn default() -> Self {
        Self {
            id: String::new(),
            color: [255; 3],
            opacity: 1000,
            base_color_texture: None,
        }
    }
}

const fn default_material_color() -> [u8; 3] {
    [255; 3]
}
const fn default_opacity() -> u16 {
    1000
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Transform3d {
    #[serde(default = "default_transform_scale")]
    pub scale: [i32; 3],
    #[serde(default)]
    pub rotation: [i32; 3],
    #[serde(default)]
    pub translation: [i32; 3],
}

impl Default for Transform3d {
    fn default() -> Self {
        Self {
            scale: [1000; 3],
            rotation: [0; 3],
            translation: [0; 3],
        }
    }
}

const fn default_transform_scale() -> [i32; 3] {
    [1000; 3]
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Object3d {
    pub id: String,
    pub mesh: String,
    pub material: String,
    #[serde(default)]
    pub transform: Transform3d,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Animation3d {
    pub id: String,
    pub duration_ticks: u64,
    #[serde(default)]
    pub looping: bool,
    #[serde(default)]
    pub tracks: Vec<AnimationTrack3d>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationTrack3d {
    pub object: String,
    #[serde(default)]
    pub keyframes: Vec<Keyframe3d>,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Keyframe3d {
    pub tick: u64,
    #[serde(default)]
    pub transform: Transform3d,
}
#[derive(Clone, Debug, Serialize)]
pub struct Capture3dManifest {
    pub schema: &'static str,
    pub scene_schema: &'static str,
    pub width: u32,
    pub height: u32,
    pub triangles: usize,
    pub visible_pixels: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<Capture3dChannel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segmentation_mapping: Option<Vec<Capture3dSegmentation>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Capture3dChannel {
    pub name: &'static str,
    pub file: String,
    pub encoding: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct Capture3dSegmentation {
    pub id: u32,
    pub triangle_id: u32,
    pub source: String,
    pub rank: u32,
}

#[derive(Clone, Debug)]
pub struct Rendered3d {
    pub color: Image,
    pub depth: Option<Vec<u16>>,
    pub normals: Option<Image>,
    pub segmentation: Option<Vec<u32>>,
    pub segmentation_mapping: Vec<Capture3dSegmentation>,
}

pub fn load(path: &std::path::Path) -> Result<Scene3d> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("scene3d_read: {}: {error}", path.display()))?;
    if metadata.len() > MAX_SCENE_BYTES {
        return Err("scene3d_too_large: plafond 1048576 octets".into());
    }
    let bytes = std::fs::read(path)
        .map_err(|error| format!("scene3d_read: {}: {error}", path.display()))?;
    let scene: Scene3d =
        serde_json::from_slice(&bytes).map_err(|error| format!("scene3d_invalid: {error}"))?;
    validate(&scene)?;
    Ok(scene)
}

pub fn load_unresolved(path: &std::path::Path) -> Result<Scene3d> {
    let metadata = std::fs::metadata(path)
        .map_err(|error| format!("scene3d_read: {}: {error}", path.display()))?;
    if metadata.len() > MAX_SCENE_BYTES {
        return Err("scene3d_too_large: plafond 1048576 octets".into());
    }
    let bytes = std::fs::read(path)
        .map_err(|error| format!("scene3d_read: {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("scene3d_invalid: {error}").into())
}

pub fn validate(scene: &Scene3d) -> Result<()> {
    if scene.schema != SCENE_SCHEMA {
        return Err(format!("scene3d_version: attendu {SCENE_SCHEMA}").into());
    }
    if !(1..=1024).contains(&scene.camera.pixels_per_unit) {
        return Err("scene3d_camera_invalid: pixels_per_unit 1..1024".into());
    }
    if scene.meshes.len() > MAX_MESHES
        || scene.materials.len() > MAX_MATERIALS
        || scene.objects.len() > MAX_OBJECTS
        || scene.animations.len() > MAX_ANIMATIONS
    {
        return Err("scene3d_resource_quota".into());
    }
    validate_animations(scene)?;
    let expanded = expanded_triangles(scene)?;
    if expanded.len() > MAX_TRIANGLES || expanded.len().saturating_mul(3) > MAX_VERTICES {
        return Err("scene3d_triangle_quota: plafond 100000".into());
    }
    let mut ids: Vec<u32> = scene.triangles.iter().map(|triangle| triangle.id).collect();
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("scene3d_duplicate_triangle: IDs uniques requis".into());
    }
    Ok(())
}

fn validate_animations(scene: &Scene3d) -> Result<()> {
    use std::collections::BTreeSet;
    let objects: BTreeSet<&str> = scene
        .objects
        .iter()
        .map(|object| object.id.as_str())
        .collect();
    let mut animation_ids = BTreeSet::new();
    let mut track_count = 0usize;
    let mut keyframe_count = 0usize;
    for animation in &scene.animations {
        if !safe_id(&animation.id) || !animation_ids.insert(animation.id.as_str()) {
            return Err("scene3d_animation_id_invalid".into());
        }
        if animation.duration_ticks == 0 {
            return Err("scene3d_animation_duration_invalid".into());
        }
        track_count = track_count
            .checked_add(animation.tracks.len())
            .ok_or("scene3d_animation_quota")?;
        let mut targets = BTreeSet::new();
        for track in &animation.tracks {
            if !objects.contains(track.object.as_str()) {
                return Err("scene3d_animation_object_reference_missing".into());
            }
            if !targets.insert(track.object.as_str()) {
                return Err("scene3d_animation_duplicate_track".into());
            }
            keyframe_count = keyframe_count
                .checked_add(track.keyframes.len())
                .ok_or("scene3d_animation_quota")?;
            let mut previous = None;
            for keyframe in &track.keyframes {
                if keyframe.tick > animation.duration_ticks
                    || previous.is_some_and(|tick| keyframe.tick <= tick)
                {
                    return Err("scene3d_keyframe_order_invalid".into());
                }
                if keyframe
                    .transform
                    .rotation
                    .iter()
                    .any(|angle| angle.rem_euclid(90_000) != 0)
                {
                    return Err(
                        "scene3d_rotation_invalid: quarts de tour en millidegres uniquement".into(),
                    );
                }
                previous = Some(keyframe.tick);
            }
        }
    }
    if track_count > MAX_TRACKS || keyframe_count > MAX_KEYFRAMES {
        return Err("scene3d_animation_quota".into());
    }
    Ok(())
}

fn safe_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

pub fn expanded_triangles(scene: &Scene3d) -> Result<Vec<Triangle>> {
    Ok(expanded_mesh_triangles(scene)?
        .into_iter()
        .map(|expanded| expanded.triangle)
        .collect())
}

/// Presentation-only expansion retaining mesh attributes which are not part of
/// the deterministic CPU triangle contract. The CPU renderer intentionally
/// consumes [`expanded_triangles`] so these optional attributes cannot alter
/// captures, checksums, or replays.
#[derive(Clone, Debug)]
pub struct ExpandedMeshTriangle3d {
    pub triangle: Triangle,
    pub normals: Option<[[i32; 3]; 3]>,
    pub uvs: Option<[[i32; 2]; 3]>,
    pub texture: Option<String>,
    pub object: Option<String>,
    pub transform: Transform3d,
}

pub fn expanded_mesh_triangles(scene: &Scene3d) -> Result<Vec<ExpandedMeshTriangle3d>> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut mesh_ids = BTreeSet::new();
    let mut meshes = BTreeMap::new();
    for mesh in &scene.meshes {
        if !safe_id(&mesh.id) || !mesh_ids.insert(mesh.id.as_str()) {
            return Err("scene3d_mesh_id_invalid".into());
        }
        if mesh.vertices.len() > MAX_VERTICES || mesh.triangles.len() > MAX_TRIANGLES {
            return Err("scene3d_mesh_quota".into());
        }
        validate_mesh_attributes(mesh)?;
        for face in &mesh.triangles {
            if face
                .iter()
                .any(|index| *index as usize >= mesh.vertices.len())
            {
                return Err("scene3d_mesh_index_invalid".into());
            }
        }
        meshes.insert(mesh.id.as_str(), mesh);
    }
    let mut material_ids = BTreeSet::new();
    let mut materials = BTreeMap::new();
    for material in &scene.materials {
        if !safe_id(&material.id) || !material_ids.insert(material.id.as_str()) {
            return Err("scene3d_material_id_invalid".into());
        }
        if material.opacity > 1000 {
            return Err("scene3d_material_opacity_invalid: attendu 0..1000".into());
        }
        if material
            .base_color_texture
            .as_deref()
            .is_some_and(|texture| !safe_id(texture))
        {
            return Err("scene3d_material_texture_id_invalid".into());
        }
        materials.insert(material.id.as_str(), material);
    }
    let mut object_ids = BTreeSet::new();
    let mut result = scene
        .triangles
        .iter()
        .cloned()
        .map(|triangle| ExpandedMeshTriangle3d {
            triangle,
            normals: None,
            uvs: None,
            texture: None,
            object: None,
            transform: Transform3d::default(),
        })
        .collect::<Vec<_>>();
    for object in &scene.objects {
        if !safe_id(&object.id) || !object_ids.insert(object.id.as_str()) {
            return Err("scene3d_object_id_invalid".into());
        }
        let mesh = meshes
            .get(object.mesh.as_str())
            .ok_or("scene3d_mesh_reference_missing")?;
        let material = materials
            .get(object.material.as_str())
            .ok_or("scene3d_material_reference_missing")?;
        if object
            .transform
            .rotation
            .iter()
            .any(|angle| angle.rem_euclid(90_000) != 0)
        {
            return Err(
                "scene3d_rotation_invalid: quarts de tour en millidegres uniquement".into(),
            );
        }
        for face in &mesh.triangles {
            let mut vertices = [Vertex3 { x: 0, y: 0, z: 0 }; 3];
            for (slot, index) in face.iter().enumerate() {
                vertices[slot] =
                    transform_vertex(mesh.vertices[*index as usize], object.transform)?;
            }
            let mut color = [0; 3];
            for (channel, output) in color.iter_mut().enumerate() {
                *output = ((u32::from(material.color[channel]) * u32::from(material.opacity)
                    + u32::from(scene.background[channel]) * (1000 - u32::from(material.opacity)))
                    / 1000) as u8;
            }
            let id = u32::try_from(result.len()).map_err(|_| "scene3d_triangle_quota")?;
            let normals =
                (!mesh.normals.is_empty()).then(|| face.map(|index| mesh.normals[index as usize]));
            let uvs = (!mesh.uvs.is_empty()).then(|| face.map(|index| mesh.uvs[index as usize]));
            result.push(ExpandedMeshTriangle3d {
                triangle: Triangle {
                    id: id.checked_add(1).ok_or("scene3d_triangle_quota")?,
                    vertices,
                    color,
                },
                normals,
                uvs,
                texture: material.base_color_texture.clone(),
                object: Some(object.id.clone()),
                transform: object.transform,
            });
            if result.len() > MAX_TRIANGLES {
                return Err("scene3d_triangle_quota: plafond 100000".into());
            }
        }
    }
    Ok(result)
}

fn validate_mesh_attributes(mesh: &Mesh3d) -> Result<()> {
    if !mesh.normals.is_empty() && mesh.normals.len() != mesh.vertices.len() {
        return Err("scene3d_mesh_normals_length_invalid".into());
    }
    if !mesh.uvs.is_empty() && mesh.uvs.len() != mesh.vertices.len() {
        return Err("scene3d_mesh_uvs_length_invalid".into());
    }
    if mesh.normals.iter().any(|normal| {
        let length_squared = normal
            .iter()
            .map(|component| i128::from(*component) * i128::from(*component))
            .sum::<i128>();
        length_squared == 0
    }) {
        return Err("scene3d_mesh_normal_invalid".into());
    }
    Ok(())
}

fn transform_vertex(vertex: Vertex3, transform: Transform3d) -> Result<Vertex3> {
    let mut value = [
        i128::from(vertex.x),
        i128::from(vertex.y),
        i128::from(vertex.z),
    ];
    for (coordinate, scale) in value.iter_mut().zip(transform.scale) {
        *coordinate = coordinate
            .checked_mul(i128::from(scale))
            .ok_or("scene3d_transform_overflow")?
            / 1000;
    }
    for axis in 0..3 {
        for _ in 0..transform.rotation[axis].rem_euclid(360_000) / 90_000 {
            value = match axis {
                0 => [value[0], -value[2], value[1]],
                1 => [value[2], value[1], -value[0]],
                _ => [-value[1], value[0], value[2]],
            };
        }
    }
    for (coordinate, translation) in value.iter_mut().zip(transform.translation) {
        *coordinate = coordinate
            .checked_add(i128::from(translation))
            .ok_or("scene3d_transform_overflow")?;
    }
    Ok(Vertex3 {
        x: i32::try_from(value[0]).map_err(|_| "scene3d_transform_overflow")?,
        y: i32::try_from(value[1]).map_err(|_| "scene3d_transform_overflow")?,
        z: i32::try_from(value[2]).map_err(|_| "scene3d_transform_overflow")?,
    })
}

pub fn resolve_assets(scene: &mut Scene3d, manifest: &std::path::Path) -> Result<()> {
    let _ = resolve_assets_with_textures(scene, manifest)?;
    Ok(())
}

/// Resolve JSON assets for the CPU path and return binary texture payloads for
/// the optional GPU path. Texture bytes never become part of `Scene3d`.
pub fn resolve_assets_with_textures(
    scene: &mut Scene3d,
    manifest: &std::path::Path,
) -> Result<std::collections::BTreeMap<String, Vec<u8>>> {
    resolve_assets_with_textures_cached(scene, manifest, None)
}

pub fn resolve_assets_with_textures_cached(
    scene: &mut Scene3d,
    manifest: &std::path::Path,
    cache_root: Option<&std::path::Path>,
) -> Result<std::collections::BTreeMap<String, Vec<u8>>> {
    let assets = match cache_root {
        Some(root) => crate::assets3d::load_manifest_cached(manifest, root)?,
        None => crate::assets3d::load_manifest(manifest)?,
    };
    let mut textures = std::collections::BTreeMap::new();
    for (_, asset) in assets {
        match asset {
            crate::assets3d::Asset3d::Mesh(mesh) => {
                if scene.meshes.iter().any(|inline| inline.id == mesh.id) {
                    return Err(format!("scene3d_mesh_collision: {}", mesh.id).into());
                }
                scene.meshes.push(mesh);
            }
            crate::assets3d::Asset3d::Material(material) => {
                if scene
                    .materials
                    .iter()
                    .any(|inline| inline.id == material.id)
                {
                    return Err(format!("scene3d_material_collision: {}", material.id).into());
                }
                scene.materials.push(material);
            }
            crate::assets3d::Asset3d::Texture(texture) => {
                textures.insert(texture.id, texture.bytes);
            }
        }
    }
    scene.meshes.sort_by(|left, right| left.id.cmp(&right.id));
    scene
        .materials
        .sort_by(|left, right| left.id.cmp(&right.id));
    validate(scene)?;
    Ok(textures)
}

fn sample_animation(scene: &Scene3d, animation_id: &str, tick: u64) -> Result<Scene3d> {
    let animation = scene
        .animations
        .iter()
        .find(|animation| animation.id == animation_id)
        .ok_or("scene3d_animation_reference_missing")?;
    let sample_tick = if animation.looping {
        tick % animation.duration_ticks
    } else {
        tick.min(animation.duration_ticks)
    };
    let mut sampled = scene.clone();
    for track in &animation.tracks {
        let Some(keyframe) = track
            .keyframes
            .iter()
            .rev()
            .find(|keyframe| keyframe.tick <= sample_tick)
            .or_else(|| track.keyframes.first())
        else {
            continue;
        };
        let object = sampled
            .objects
            .iter_mut()
            .find(|object| object.id == track.object)
            .ok_or("scene3d_animation_object_reference_missing")?;
        object.transform = keyframe.transform;
    }
    Ok(sampled)
}

pub fn capture(
    scene_path: &std::path::Path,
    output: &std::path::Path,
    width: u32,
    height: u32,
    ticks: u64,
    animation: Option<&str>,
) -> Result<std::path::PathBuf> {
    capture_with_assets(
        scene_path,
        None,
        output,
        width,
        height,
        ticks,
        animation,
        &Channels::default(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn capture_with_assets(
    scene_path: &std::path::Path,
    assets_path: Option<&std::path::Path>,
    output: &std::path::Path,
    width: u32,
    height: u32,
    ticks: u64,
    animation: Option<&str>,
    channels: &Channels,
) -> Result<std::path::PathBuf> {
    capture_with_assets_cached(
        scene_path,
        assets_path,
        output,
        width,
        height,
        ticks,
        animation,
        channels,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn capture_with_assets_cached(
    scene_path: &std::path::Path,
    assets_path: Option<&std::path::Path>,
    output: &std::path::Path,
    width: u32,
    height: u32,
    ticks: u64,
    animation: Option<&str>,
    channels: &Channels,
    cache_root: Option<&std::path::Path>,
) -> Result<std::path::PathBuf> {
    let mut scene = if assets_path.is_some() {
        load_unresolved(scene_path)?
    } else {
        load(scene_path)?
    };
    if let Some(path) = assets_path {
        let _ = resolve_assets_with_textures_cached(&mut scene, path, cache_root)?;
    }
    let sampled = animation
        .map(|id| sample_animation(&scene, id, ticks))
        .transpose()?;
    let render_scene = sampled.as_ref().unwrap_or(&scene);
    let (buffers, mut manifest) = render_buffers(render_scene, width, height)?;
    if let Some(id) = animation {
        manifest.animation = Some(id.to_owned());
        manifest.tick = Some(ticks);
    }
    let batch = crate::render3d_channels::build(
        output,
        &buffers,
        channels,
        &manifest.segmentation_mapping.clone().unwrap_or_default(),
    )?;
    manifest.channels = batch.manifest;
    manifest.segmentation_mapping = batch.segmentation_mapping;
    let image_bytes = crate::render::encode_ppm(&buffers.color);
    let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("capture3d_manifest_invalid: {error}"))?;
    manifest_bytes.push(b'\n');
    let manifest_path = crate::render::manifest_path(output);
    crate::render3d_channels::publish_atomic(
        output,
        &image_bytes,
        &manifest_path,
        &manifest_bytes,
        &batch.files,
    )?;
    Ok(manifest_path)
}

pub fn render(scene: &Scene3d, width: u32, height: u32) -> Result<(Image, Capture3dManifest)> {
    let (buffers, manifest) = render_buffers(scene, width, height)?;
    Ok((buffers.color, manifest))
}

fn render_buffers(
    scene: &Scene3d,
    width: u32,
    height: u32,
) -> Result<(RenderBuffers, Capture3dManifest)> {
    validate(scene)?;
    let pixel_count = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or("scene3d_dimensions_invalid")?;
    if width == 0 || height == 0 || pixel_count > MAX_PIXELS {
        return Err("scene3d_dimensions_invalid: plafond 16777216 pixels".into());
    }
    let pixel_count = usize::try_from(pixel_count).map_err(|_| "scene3d_dimensions_invalid")?;
    let mut image = Image {
        width,
        height,
        pixels: vec![0; pixel_count * 3],
    };
    for pixel in image.pixels.chunks_exact_mut(3) {
        pixel.copy_from_slice(&scene.background);
    }
    let mut depth = vec![i64::MAX; pixel_count];
    let mut owner = vec![u32::MAX; pixel_count];
    let mut triangles = expanded_triangles(scene)?;
    triangles.sort_by_key(|triangle| triangle.id);
    for triangle in &triangles {
        rasterize(&mut image, &mut depth, &mut owner, triangle, scene.camera)?;
    }
    let triangle_count = triangles.len();
    let visible_pixels = owner.iter().filter(|value| **value != u32::MAX).count() as u64;
    let mut depth_channel = vec![u16::MAX; pixel_count];
    let mut normals = Image {
        width,
        height,
        pixels: vec![0; pixel_count * 3],
    };
    let mut segmentation = vec![0; pixel_count];
    for pixel in 0..pixel_count {
        if owner[pixel] != u32::MAX {
            let relative = depth[pixel].saturating_sub(i64::from(scene.camera.z));
            depth_channel[pixel] = u16::try_from(relative.clamp(0, i64::from(u16::MAX)))
                .map_err(|_| "scene3d_depth_overflow")?;
            normals.pixels[pixel * 3..pixel * 3 + 3].copy_from_slice(&[128, 128, 255]);
            segmentation[pixel] = owner[pixel]
                .checked_add(1)
                .ok_or("scene3d_segmentation_overflow")?;
        }
    }
    let segmentation_mapping = triangles
        .iter()
        .enumerate()
        .map(|(rank, triangle)| Capture3dSegmentation {
            id: triangle.id + 1,
            triangle_id: triangle.id,
            source: format!("triangle:{}", triangle.id),
            rank: rank as u32,
        })
        .collect();
    Ok((
        RenderBuffers {
            color: image,
            depth: depth_channel,
            normals,
            segmentation,
            segmentation_mapping: Vec::new(),
        },
        Capture3dManifest {
            schema: CAPTURE_SCHEMA,
            scene_schema: SCENE_SCHEMA,
            width,
            height,
            triangles: triangle_count,
            visible_pixels,
            animation: None,
            tick: None,
            channels: None,
            segmentation_mapping: Some(segmentation_mapping),
        },
    ))
}

fn rasterize(
    image: &mut Image,
    depth: &mut [i64],
    owner: &mut [u32],
    triangle: &Triangle,
    camera: Camera3d,
) -> Result<()> {
    let mut points = [(0_i64, 0_i64, 0_i64); 3];
    for (index, vertex) in triangle.vertices.iter().enumerate() {
        let x = i64::from(vertex.x)
            .checked_sub(i64::from(camera.x))
            .and_then(|value| value.checked_mul(i64::from(camera.pixels_per_unit)))
            .and_then(|value| value.checked_add(i64::from(image.width / 2)))
            .ok_or("scene3d_coordinate_overflow")?;
        let y = i64::from(image.height / 2)
            .checked_sub(
                i64::from(vertex.y)
                    .checked_sub(i64::from(camera.y))
                    .and_then(|value| value.checked_mul(i64::from(camera.pixels_per_unit)))
                    .ok_or("scene3d_coordinate_overflow")?,
            )
            .ok_or("scene3d_coordinate_overflow")?;
        let z = i64::from(vertex.z) - i64::from(camera.z);
        points[index] = (x, y, z);
    }
    let area = edge(points[0], points[1], points[2].0, points[2].1)?;
    if area == 0 {
        return Ok(());
    }
    let min_x = points.iter().map(|point| point.0).min().unwrap().max(0);
    let max_x = points
        .iter()
        .map(|point| point.0)
        .max()
        .unwrap()
        .min(i64::from(image.width) - 1);
    let min_y = points.iter().map(|point| point.1).min().unwrap().max(0);
    let max_y = points
        .iter()
        .map(|point| point.1)
        .max()
        .unwrap()
        .min(i64::from(image.height) - 1);
    if min_x > max_x || min_y > max_y {
        return Ok(());
    }
    let sign = area.signum();
    let denominator = i128::from(area.abs());
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let w0 = edge(points[1], points[2], x, y)? * sign;
            let w1 = edge(points[2], points[0], x, y)? * sign;
            let w2 = edge(points[0], points[1], x, y)? * sign;
            if w0 < 0 || w1 < 0 || w2 < 0 {
                continue;
            }
            let numerator = i128::from(w0) * i128::from(points[0].2)
                + i128::from(w1) * i128::from(points[1].2)
                + i128::from(w2) * i128::from(points[2].2);
            let z = i64::try_from(numerator / denominator).map_err(|_| "scene3d_depth_overflow")?;
            let pixel = y as usize * image.width as usize + x as usize;
            if z < depth[pixel] || (z == depth[pixel] && triangle.id < owner[pixel]) {
                depth[pixel] = z;
                owner[pixel] = triangle.id;
                image.pixels[pixel * 3..pixel * 3 + 3].copy_from_slice(&triangle.color);
            }
        }
    }
    Ok(())
}

fn edge(a: (i64, i64, i64), b: (i64, i64, i64), x: i64, y: i64) -> Result<i64> {
    let value =
        i128::from(x - a.0) * i128::from(b.1 - a.1) - i128::from(y - a.1) * i128::from(b.0 - a.0);
    i64::try_from(value).map_err(|_| "scene3d_coordinate_overflow".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(triangles: Vec<Triangle>) -> Scene3d {
        Scene3d {
            schema: SCENE_SCHEMA.into(),
            camera: Camera3d {
                pixels_per_unit: 1,
                ..Camera3d::default()
            },
            background: [1, 2, 3],
            triangles,
            meshes: Vec::new(),
            materials: Vec::new(),
            objects: Vec::new(),
            animations: Vec::new(),
        }
    }

    fn triangle(id: u32, z: i32, color: [u8; 3]) -> Triangle {
        Triangle {
            id,
            vertices: [
                Vertex3 { x: -2, y: -2, z },
                Vertex3 { x: 2, y: -2, z },
                Vertex3 { x: 0, y: 2, z },
            ],
            color,
        }
    }

    #[test]
    fn rendering_is_independent_of_declaration_order() {
        let near = triangle(2, 1, [255, 0, 0]);
        let far = triangle(1, 5, [0, 0, 255]);
        let first = render(&scene(vec![near.clone(), far.clone()]), 8, 8)
            .unwrap()
            .0;
        let second = render(&scene(vec![far, near]), 8, 8).unwrap().0;
        assert_eq!(first, second);
        assert_eq!(
            &first.pixels[(4 * 8 + 4) * 3..(4 * 8 + 4) * 3 + 3],
            &[255, 0, 0]
        );
    }

    #[test]
    fn equal_depth_uses_triangle_id_as_stable_tie_breaker() {
        let low = triangle(1, 2, [10, 20, 30]);
        let high = triangle(2, 2, [40, 50, 60]);
        let image = render(&scene(vec![high, low]), 8, 8).unwrap().0;
        assert_eq!(
            &image.pixels[(4 * 8 + 4) * 3..(4 * 8 + 4) * 3 + 3],
            &[10, 20, 30]
        );
    }

    #[test]
    fn strict_version_duplicates_and_limits_are_rejected() {
        let mut value = scene(vec![triangle(1, 0, [0, 0, 0]), triangle(1, 1, [1, 1, 1])]);
        assert!(validate(&value).is_err());
        value.schema = "aetherion.scene3d/v2".into();
        assert!(validate(&value).is_err());
        assert!(render(&scene(Vec::new()), 0, 1).is_err());
    }

    fn resource_scene(objects: Vec<Object3d>) -> Scene3d {
        Scene3d {
            meshes: vec![Mesh3d {
                id: "mesh".into(),
                vertices: vec![
                    Vertex3 { x: 1, y: 2, z: 3 },
                    Vertex3 { x: 2, y: 2, z: 3 },
                    Vertex3 { x: 1, y: 3, z: 3 },
                ],
                triangles: vec![[0, 1, 2]],
                normals: Vec::new(),
                uvs: Vec::new(),
            }],
            materials: vec![Material3d {
                id: "material".into(),
                color: [201, 101, 51],
                opacity: 500,
                base_color_texture: None,
            }],
            objects,
            ..scene(Vec::new())
        }
    }

    fn object(id: &str) -> Object3d {
        Object3d {
            id: id.into(),
            mesh: "mesh".into(),
            material: "material".into(),
            transform: Transform3d::default(),
        }
    }

    #[test]
    fn legacy_triangles_remain_compatible_with_empty_resources() {
        let legacy = scene(vec![triangle(7, 1, [9, 8, 7])]);
        let (image, manifest) = render(&legacy, 8, 8).unwrap();
        assert_eq!(manifest.triangles, 1);
        assert_eq!(
            &image.pixels[(4 * 8 + 4) * 3..(4 * 8 + 4) * 3 + 3],
            &[9, 8, 7]
        );
    }

    #[test]
    fn transform_order_is_scale_rx_ry_rz_then_translation() {
        let transform = Transform3d {
            scale: [2000, 3000, 4000],
            rotation: [90_000, 90_000, 90_000],
            translation: [10, 20, 30],
        };
        let transformed = transform_vertex(Vertex3 { x: 1, y: 2, z: 3 }, transform).unwrap();
        assert_eq!([transformed.x, transformed.y, transformed.z], [22, 26, 28]);
    }

    #[test]
    fn meshes_and_materials_are_reusable_and_opacity_is_integer_deterministic() {
        let expanded = expanded_triangles(&resource_scene(vec![object("a"), object("b")])).unwrap();
        assert_eq!(expanded.len(), 2);
        for (left, right) in expanded[0].vertices.iter().zip(expanded[1].vertices) {
            assert_eq!([left.x, left.y, left.z], [right.x, right.y, right.z]);
        }
        assert_eq!(expanded[0].color, [101, 51, 27]);
        assert_eq!(expanded[0].color, expanded[1].color);
    }

    #[test]
    fn mesh_attributes_are_optional_but_length_checked() {
        let mut scene = resource_scene(vec![object("object")]);
        scene.meshes[0].normals = vec![[0, 0, NORMAL_SCALE]; 2];
        assert!(
            validate(&scene)
                .unwrap_err()
                .to_string()
                .contains("normals_length")
        );
        scene.meshes[0].normals = vec![[0, 0, NORMAL_SCALE]; 3];
        scene.meshes[0].uvs = vec![[0, 0], [UV_SCALE, 0], [0, UV_SCALE]];
        validate(&scene).unwrap();
        let expanded = expanded_mesh_triangles(&scene).unwrap();
        assert_eq!(expanded[0].normals, Some([[0, 0, NORMAL_SCALE]; 3]));
        assert_eq!(
            expanded[0].uvs,
            Some([[0, 0], [UV_SCALE, 0], [0, UV_SCALE]])
        );
    }

    #[test]
    fn legacy_mesh_without_attributes_keeps_flat_fallback() {
        let expanded = expanded_mesh_triangles(&resource_scene(vec![object("object")])).unwrap();
        assert_eq!(expanded[0].normals, None);
        assert_eq!(expanded[0].uvs, None);
        assert_eq!(
            expanded_triangles(&resource_scene(vec![object("object")]))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn cpu_render_ignores_external_base_color_texture() {
        let plain = resource_scene(vec![object("object")]);
        let mut textured = plain.clone();
        textured.materials[0].base_color_texture = Some("albedo".into());
        assert_eq!(
            render(&plain, 16, 16).unwrap().0,
            render(&textured, 16, 16).unwrap().0
        );
    }

    #[test]
    fn mesh_attributes_have_stable_json_representation() {
        let scene = resource_scene(vec![object("object")]);
        let first = serde_json::to_vec(&scene).unwrap();
        let second =
            serde_json::to_vec(&serde_json::from_slice::<Scene3d>(&first).unwrap()).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn invalid_references_indices_and_rotations_are_rejected() {
        let mut missing_mesh = resource_scene(vec![object("object")]);
        missing_mesh.objects[0].mesh = "absent".into();
        assert!(
            validate(&missing_mesh)
                .unwrap_err()
                .to_string()
                .contains("mesh_reference")
        );

        let mut missing_material = resource_scene(vec![object("object")]);
        missing_material.objects[0].material = "absent".into();
        assert!(
            validate(&missing_material)
                .unwrap_err()
                .to_string()
                .contains("material_reference")
        );

        let mut bad_index = resource_scene(vec![object("object")]);
        bad_index.meshes[0].triangles[0][2] = 3;
        assert!(
            validate(&bad_index)
                .unwrap_err()
                .to_string()
                .contains("mesh_index")
        );

        let mut bad_rotation = resource_scene(vec![object("object")]);
        bad_rotation.objects[0].transform.rotation = [1, 0, 0];
        assert!(
            validate(&bad_rotation)
                .unwrap_err()
                .to_string()
                .contains("rotation_invalid")
        );
    }

    #[test]
    fn canonical_triangle_order_is_independent_of_object_declaration_order() {
        let mut left = object("left");
        left.transform.translation = [-1, 0, 0];
        let mut right = object("right");
        right.transform.translation = [1, 0, 0];
        let first = render(&resource_scene(vec![left.clone(), right.clone()]), 16, 16)
            .unwrap()
            .0;
        let second = render(&resource_scene(vec![right, left]), 16, 16)
            .unwrap()
            .0;
        assert_eq!(first, second);
    }
}
