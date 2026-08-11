use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::Result;
use crate::assets::Texture;
use crate::project::{AnimationMode, AtlasRegion, RenderConfig, SpriteConfig};
use crate::simulation::{EntityState, World};

#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderBuffers {
    pub color: Image,
    pub depth: Vec<u16>,
    pub normals: Image,
    pub segmentation: Vec<u32>,
    pub segmentation_mapping: Vec<SegmentationEntry>,
}

pub type Framebuffer = RenderBuffers;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SegmentationEntry {
    pub value: u32,
    pub entity_id: u64,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CaptureManifest {
    pub schema: &'static str,
    pub project: String,
    pub tick: u64,
    pub world_checksum: u64,
    pub image_checksum: u64,
    pub path: String,
    pub format: &'static str,
    pub dimensions: Dimensions,
    pub camera: CameraManifest,
    pub visible_entities: Vec<VisibleEntity>,
}

#[derive(Debug, Serialize)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize)]
pub struct CameraManifest {
    pub x: i64,
    pub y: i64,
    pub pixels_per_unit: u32,
}

#[derive(Debug, Serialize)]
pub struct VisibleEntity {
    pub id: u64,
    pub name: String,
    pub world_position: Point,
    pub screen_bounds: Bounds,
    pub color: [u8; 3],
}

#[derive(Debug, Serialize)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Serialize)]
pub struct Bounds {
    pub x: i64,
    pub y: i64,
    pub width: u32,
    pub height: u32,
}

/// Renders the world with no resolved textures. Identical to the pre-M4
/// renderer: every entity is drawn as a flat colored rectangle.
pub fn render(world: &World, config: &RenderConfig) -> Result<(Image, Vec<VisibleEntity>)> {
    render_with_textures(world, config, &BTreeMap::new())
}

/// Renders the world into an RGB framebuffer.
///
/// Entities that carry a visible [`SpriteConfig`] whose asset is present in
/// `textures` are rasterized as textures (nearest-neighbor scaled, alpha
/// composited, honoring flip and tint). Every other entity falls back to the
/// historical flat-rectangle behavior. When `textures` is empty the output is
/// byte-identical to the pre-M4 renderer.
pub fn render_with_textures(
    world: &World,
    config: &RenderConfig,
    textures: &BTreeMap<String, Texture>,
) -> Result<(Image, Vec<VisibleEntity>)> {
    let (buffers, visible) = render_buffers_with_textures(world, config, textures)?;
    Ok((buffers.color, visible))
}

pub fn render_buffers_with_textures(
    world: &World,
    config: &RenderConfig,
    textures: &BTreeMap<String, Texture>,
) -> Result<(RenderBuffers, Vec<VisibleEntity>)> {
    let pixels = (config.width as usize)
        .checked_mul(config.height as usize)
        .ok_or("dimensions de rendu trop grandes")?;
    let rgb = pixels
        .checked_mul(3)
        .ok_or("dimensions de rendu trop grandes")?;
    let mut buffers = RenderBuffers {
        color: Image {
            width: config.width,
            height: config.height,
            pixels: vec![0; rgb],
        },
        depth: vec![0; pixels],
        normals: Image {
            width: config.width,
            height: config.height,
            pixels: vec![0; rgb],
        },
        segmentation: vec![0; pixels],
        segmentation_mapping: Vec::new(),
    };
    for pixel in buffers.color.pixels.chunks_exact_mut(3) {
        pixel.copy_from_slice(&config.background);
    }
    let mut drawables = Vec::new();
    for entity in world.entities() {
        let sprite = world.sprite(entity.id);
        let use_sprite =
            sprite.is_some_and(|value| value.visible && textures.contains_key(&value.asset));
        let z = if use_sprite {
            sprite.map(|value| value.z).unwrap_or(0)
        } else {
            0
        };
        drawables.push(Drawable {
            entity,
            use_sprite,
            z,
        });
    }
    drawables.sort_by(|a, b| a.z.cmp(&b.z).then(a.entity.id.cmp(&b.entity.id)));
    let mut visible = Vec::new();
    for (rank, drawable) in drawables.iter().enumerate() {
        let entity = &drawable.entity;
        let segmentation = u32::try_from(rank + 1).map_err(|_| "trop d'entités à segmenter")?;
        let depth = u16::try_from((rank + 1).min(u16::MAX as usize)).unwrap_or(u16::MAX);
        buffers.segmentation_mapping.push(SegmentationEntry {
            value: segmentation,
            entity_id: entity.id,
            name: entity.name.clone(),
        });
        if drawable.use_sprite {
            let sprite = world.sprite(entity.id).expect("sprite présent");
            let texture = textures.get(&sprite.asset).expect("texture présente");
            let region = resolve_animation_region(sprite, world.tick)
                .unwrap_or_else(|| full_region(texture));
            if let Some(bounds) = sprite_bounds(entity, sprite, config)? {
                draw_texture_buffers(
                    &mut buffers,
                    &bounds,
                    texture,
                    &region,
                    sprite,
                    depth,
                    segmentation,
                );
                visible.push(VisibleEntity {
                    id: entity.id,
                    name: entity.name.clone(),
                    world_position: Point {
                        x: entity.position.x,
                        y: entity.position.y,
                    },
                    screen_bounds: bounds,
                    color: [sprite.tint[0], sprite.tint[1], sprite.tint[2]],
                });
            }
        } else if let Some(bounds) = entity_bounds(entity, config)? {
            draw_rectangle_buffers(
                &mut buffers,
                &bounds,
                entity.appearance.color,
                depth,
                segmentation,
            );
            visible.push(VisibleEntity {
                id: entity.id,
                name: entity.name.clone(),
                world_position: Point {
                    x: entity.position.x,
                    y: entity.position.y,
                },
                screen_bounds: bounds,
                color: entity.appearance.color,
            });
        }
    }
    Ok((buffers, visible))
}

struct Drawable {
    entity: EntityState,
    use_sprite: bool,
    z: i32,
}

fn full_region(texture: &Texture) -> AtlasRegion {
    AtlasRegion {
        x: 0,
        y: 0,
        width: texture.width,
        height: texture.height,
    }
}

/// Resolves the active atlas region for a sprite at the given tick.
///
/// When the sprite has no animation this returns its static `region` (or
/// `None`, meaning "use the full texture"). With an animation, frame
/// `duration_ticks` are accumulated deterministically: `Loop` wraps around the
/// total cycle length, while `Once` clamps to the final frame. Purely integer
/// arithmetic, so identical ticks always produce identical frames.
pub fn resolve_animation_region(sprite: &SpriteConfig, tick: u64) -> Option<AtlasRegion> {
    let Some(animation) = &sprite.animation else {
        return sprite.region;
    };
    if animation.frames.is_empty() {
        return sprite.region;
    }
    let total: u64 = animation
        .frames
        .iter()
        .map(|frame| u64::from(frame.duration_ticks))
        .sum();
    if total == 0 {
        return Some(animation.frames[0].region);
    }
    let position = match animation.mode {
        AnimationMode::Loop => tick % total,
        AnimationMode::Once => {
            if tick >= total {
                // Clamp to the last frame's slot.
                total - 1
            } else {
                tick
            }
        }
    };
    let mut cursor = 0_u64;
    for frame in &animation.frames {
        cursor += u64::from(frame.duration_ticks);
        if position < cursor {
            return Some(frame.region);
        }
    }
    animation.frames.last().map(|frame| frame.region)
}

fn entity_bounds(entity: &EntityState, config: &RenderConfig) -> Result<Option<Bounds>> {
    let scale = i64::from(config.camera.pixels_per_unit);
    let center_x = i64::from(config.width / 2)
        .checked_add(
            entity
                .position
                .x
                .checked_sub(config.camera.x)
                .and_then(|value| value.checked_mul(scale))
                .ok_or("coordonnee X hors limite pendant le rendu")?,
        )
        .ok_or("coordonnee X hors limite pendant le rendu")?;
    let center_y = i64::from(config.height / 2)
        .checked_sub(
            entity
                .position
                .y
                .checked_sub(config.camera.y)
                .and_then(|value| value.checked_mul(scale))
                .ok_or("coordonnee Y hors limite pendant le rendu")?,
        )
        .ok_or("coordonnee Y hors limite pendant le rendu")?;
    let width = entity
        .appearance
        .width
        .checked_mul(config.camera.pixels_per_unit)
        .ok_or("largeur d'entite hors limite")?;
    let height = entity
        .appearance
        .height
        .checked_mul(config.camera.pixels_per_unit)
        .ok_or("hauteur d'entite hors limite")?;
    let x = center_x - i64::from(width / 2);
    let y = center_y - i64::from(height / 2);
    if x >= i64::from(config.width)
        || y >= i64::from(config.height)
        || x + i64::from(width) <= 0
        || y + i64::from(height) <= 0
    {
        return Ok(None);
    }
    Ok(Some(Bounds {
        x,
        y,
        width,
        height,
    }))
}

/// Computes the destination bounds for a sprite. The sprite occupies the same
/// footprint as the entity appearance (width/height in world units scaled by
/// `pixels_per_unit`), shifted by the pivot (a pixel offset). A pivot of (0, 0)
/// reproduces the centered placement used for flat rectangles.
fn sprite_bounds(
    entity: &EntityState,
    sprite: &SpriteConfig,
    config: &RenderConfig,
) -> Result<Option<Bounds>> {
    let scale = i64::from(config.camera.pixels_per_unit);
    let center_x = i64::from(config.width / 2)
        .checked_add(
            entity
                .position
                .x
                .checked_sub(config.camera.x)
                .and_then(|value| value.checked_mul(scale))
                .ok_or("coordonnee X hors limite pendant le rendu")?,
        )
        .ok_or("coordonnee X hors limite pendant le rendu")?;
    let center_y = i64::from(config.height / 2)
        .checked_sub(
            entity
                .position
                .y
                .checked_sub(config.camera.y)
                .and_then(|value| value.checked_mul(scale))
                .ok_or("coordonnee Y hors limite pendant le rendu")?,
        )
        .ok_or("coordonnee Y hors limite pendant le rendu")?;
    let width = entity
        .appearance
        .width
        .checked_mul(config.camera.pixels_per_unit)
        .ok_or("largeur d'entite hors limite")?;
    let height = entity
        .appearance
        .height
        .checked_mul(config.camera.pixels_per_unit)
        .ok_or("hauteur d'entite hors limite")?;
    let x = center_x - i64::from(width / 2) - i64::from(sprite.pivot.x);
    let y = center_y - i64::from(height / 2) - i64::from(sprite.pivot.y);
    if x >= i64::from(config.width)
        || y >= i64::from(config.height)
        || x + i64::from(width) <= 0
        || y + i64::from(height) <= 0
    {
        return Ok(None);
    }
    Ok(Some(Bounds {
        x,
        y,
        width,
        height,
    }))
}

fn draw_rectangle_buffers(
    buffers: &mut RenderBuffers,
    bounds: &Bounds,
    color: [u8; 3],
    depth: u16,
    segmentation: u32,
) {
    let image = &mut buffers.color;
    let min_x = bounds.x.max(0) as u32;
    let min_y = bounds.y.max(0) as u32;
    let max_x = (bounds.x + i64::from(bounds.width)).min(i64::from(image.width)) as u32;
    let max_y = (bounds.y + i64::from(bounds.height)).min(i64::from(image.height)) as u32;
    for y in min_y..max_y {
        for x in min_x..max_x {
            let offset = ((y * image.width + x) * 3) as usize;
            image.pixels[offset..offset + 3].copy_from_slice(&color);
            let pixel = (y * image.width + x) as usize;
            buffers.depth[pixel] = depth;
            buffers.normals.pixels[pixel * 3..pixel * 3 + 3].copy_from_slice(&[128, 128, 255]);
            buffers.segmentation[pixel] = segmentation;
        }
    }
}

/// Blends one 8-bit channel of `src` over `dst` given coverage `alpha`.
/// `out = round((src*alpha + dst*(255-alpha)) / 255)`.
fn blend_over(src: u8, dst: u8, alpha: u8) -> u8 {
    let alpha = u32::from(alpha);
    let value = u32::from(src) * alpha + u32::from(dst) * (255 - alpha) + 127;
    (value / 255) as u8
}

/// Multiplies two 8-bit channels: `round((a*b)/255)`.
fn modulate(a: u8, b: u8) -> u8 {
    ((u32::from(a) * u32::from(b) + 127) / 255) as u8
}

/// Rasterizes `texture` (restricted to `region`) into `bounds` using
/// nearest-neighbor integer scaling, honoring `flip_x`/`flip_y`, applying the
/// sprite `tint` (RGB modulate + alpha modulate), then alpha-compositing the
/// resulting RGBA texel over the RGB framebuffer. All arithmetic is integer and
/// deterministic.
#[cfg(test)]
fn draw_texture(
    image: &mut Image,
    bounds: &Bounds,
    texture: &Texture,
    region: &AtlasRegion,
    sprite: &SpriteConfig,
) {
    let pixels = image.width as usize * image.height as usize;
    let mut buffers = RenderBuffers {
        color: image.clone(),
        depth: vec![0; pixels],
        normals: Image {
            width: image.width,
            height: image.height,
            pixels: vec![0; pixels * 3],
        },
        segmentation: vec![0; pixels],
        segmentation_mapping: Vec::new(),
    };
    draw_texture_buffers(&mut buffers, bounds, texture, region, sprite, 1, 1);
    *image = buffers.color;
}

fn draw_texture_buffers(
    buffers: &mut RenderBuffers,
    bounds: &Bounds,
    texture: &Texture,
    region: &AtlasRegion,
    sprite: &SpriteConfig,
    depth: u16,
    segmentation: u32,
) {
    let image = &mut buffers.color;
    if bounds.width == 0 || bounds.height == 0 || region.width == 0 || region.height == 0 {
        return;
    }
    // Clamp the region to the texture so a malformed atlas cannot read OOB.
    let region_x = region.x.min(texture.width);
    let region_y = region.y.min(texture.height);
    let region_w = region.width.min(texture.width - region_x);
    let region_h = region.height.min(texture.height - region_y);
    if region_w == 0 || region_h == 0 {
        return;
    }

    let min_x = bounds.x.max(0);
    let min_y = bounds.y.max(0);
    let max_x = (bounds.x + i64::from(bounds.width)).min(i64::from(image.width));
    let max_y = (bounds.y + i64::from(bounds.height)).min(i64::from(image.height));

    for py in min_y..max_y {
        // Destination pixel offset within the sprite footprint.
        let dy = (py - bounds.y) as u64;
        let mut sy = (dy * u64::from(region_h)) / u64::from(bounds.height);
        if sy >= u64::from(region_h) {
            sy = u64::from(region_h) - 1;
        }
        if sprite.flip_y {
            sy = u64::from(region_h) - 1 - sy;
        }
        let tex_row = u64::from(region_y) + sy;
        for px in min_x..max_x {
            let dx = (px - bounds.x) as u64;
            let mut sx = (dx * u64::from(region_w)) / u64::from(bounds.width);
            if sx >= u64::from(region_w) {
                sx = u64::from(region_w) - 1;
            }
            if sprite.flip_x {
                sx = u64::from(region_w) - 1 - sx;
            }
            let tex_col = u64::from(region_x) + sx;
            let texel = ((tex_row * u64::from(texture.width) + tex_col) * 4) as usize;
            let tr = texture.rgba[texel];
            let tg = texture.rgba[texel + 1];
            let tb = texture.rgba[texel + 2];
            let ta = texture.rgba[texel + 3];
            // Tint: modulate RGB by tint RGB, and texel alpha by tint alpha.
            let sr = modulate(tr, sprite.tint[0]);
            let sg = modulate(tg, sprite.tint[1]);
            let sb = modulate(tb, sprite.tint[2]);
            let sa = modulate(ta, sprite.tint[3]);
            if sa == 0 {
                continue;
            }
            let offset = ((py as u32 * image.width + px as u32) * 3) as usize;
            image.pixels[offset] = blend_over(sr, image.pixels[offset], sa);
            image.pixels[offset + 1] = blend_over(sg, image.pixels[offset + 1], sa);
            image.pixels[offset + 2] = blend_over(sb, image.pixels[offset + 2], sa);
            let pixel = py as usize * image.width as usize + px as usize;
            buffers.depth[pixel] = depth;
            buffers.normals.pixels[pixel * 3..pixel * 3 + 3].copy_from_slice(&[128, 128, 255]);
            buffers.segmentation[pixel] = segmentation;
        }
    }
}

pub fn capture(world: &World, config: &RenderConfig, output: &Path) -> Result<PathBuf> {
    let (image, visible_entities) = render(world, config)?;
    let ppm = encode_ppm(&image);
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| format!("creation de {}: {e}", parent.display()))?;
    }
    fs::write(output, &ppm).map_err(|e| format!("ecriture de {}: {e}", output.display()))?;

    let manifest_path = manifest_path(output);
    let manifest = CaptureManifest {
        schema: "aetherion.capture/v1",
        project: world.project.clone(),
        tick: world.tick,
        world_checksum: world.checksum(),
        image_checksum: checksum_bytes(&ppm),
        path: output.to_string_lossy().replace('\\', "/"),
        format: "ppm-p6",
        dimensions: Dimensions {
            width: image.width,
            height: image.height,
        },
        camera: CameraManifest {
            x: config.camera.x,
            y: config.camera.y,
            pixels_per_unit: config.camera.pixels_per_unit,
        },
        visible_entities,
    };
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("serialisation du manifeste: {e}"))?;
    fs::write(&manifest_path, format!("{json}\n"))
        .map_err(|e| format!("ecriture de {}: {e}", manifest_path.display()))?;
    Ok(manifest_path)
}

pub fn encode_ppm(image: &Image) -> Vec<u8> {
    let mut bytes = format!("P6\n{} {}\n255\n", image.width, image.height).into_bytes();
    bytes.extend_from_slice(&image.pixels);
    bytes
}

pub fn manifest_path(output: &Path) -> PathBuf {
    let mut value = output.as_os_str().to_os_string();
    value.push(".json");
    PathBuf::from(value)
}

pub fn checksum_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{
        AnimationConfig, AnimationFrame, AnimationMode, AtlasRegion, Pivot, Project, SpriteConfig,
    };

    fn fixture() -> (World, RenderConfig) {
        let project: Project = toml::from_str(Project::example()).unwrap();
        let config = project.render.clone();
        (World::from_project(project), config)
    }

    fn solid_texture(rgba: [u8; 4]) -> Texture {
        Texture {
            width: 1,
            height: 1,
            rgba: rgba.to_vec(),
        }
    }

    fn sprite(asset: &str) -> SpriteConfig {
        SpriteConfig {
            asset: asset.to_string(),
            region: None,
            pivot: Pivot { x: 0, y: 0 },
            z: 0,
            visible: true,
            tint: [255, 255, 255, 255],
            flip_x: false,
            flip_y: false,
            animation: None,
        }
    }

    #[test]
    fn center_pixel_has_player_color() {
        let (world, config) = fixture();
        let (image, visible) = render(&world, &config).unwrap();
        let offset = (((image.height / 2) * image.width + image.width / 2) * 3) as usize;
        assert_eq!(&image.pixels[offset..offset + 3], &[80, 220, 120]);
        assert_eq!(visible[0].name, "player");
    }

    #[test]
    fn identical_inputs_produce_identical_ppm() {
        let (world, config) = fixture();
        let first = encode_ppm(&render(&world, &config).unwrap().0);
        let second = encode_ppm(&render(&world, &config).unwrap().0);
        assert_eq!(first, second);
        assert_eq!(checksum_bytes(&first), checksum_bytes(&second));
    }

    #[test]
    fn empty_texture_map_matches_flat_render() {
        let (world, config) = fixture();
        let baseline = render(&world, &config).unwrap().0;
        let with_empty = render_with_textures(&world, &config, &BTreeMap::new())
            .unwrap()
            .0;
        assert_eq!(baseline.pixels, with_empty.pixels);
    }

    #[test]
    fn alpha_compositing_opaque_half_and_zero() {
        // opaque source fully replaces the background
        assert_eq!(blend_over(200, 40, 255), 200);
        // zero alpha leaves background unchanged
        assert_eq!(blend_over(200, 40, 0), 40);
        // 50% alpha blends: round((200*128 + 40*127 + 127)/255) = 121
        assert_eq!(blend_over(200, 40, 128), 120);
    }

    #[test]
    fn draw_texture_composites_over_background() {
        let mut image = Image {
            width: 1,
            height: 1,
            pixels: vec![40, 40, 40],
        };
        let bounds = Bounds {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        };
        // Opaque red texel fully replaces the background.
        let texture = solid_texture([200, 10, 20, 255]);
        let region = full_region(&texture);
        draw_texture(&mut image, &bounds, &texture, &region, &sprite("t"));
        assert_eq!(image.pixels, vec![200, 10, 20]);

        // Fully transparent texel leaves the background untouched.
        let mut image = Image {
            width: 1,
            height: 1,
            pixels: vec![40, 40, 40],
        };
        let clear = solid_texture([200, 10, 20, 0]);
        draw_texture(&mut image, &bounds, &clear, &region, &sprite("t"));
        assert_eq!(image.pixels, vec![40, 40, 40]);
    }

    #[test]
    fn flip_x_and_flip_y_mirror_sampling() {
        // 2x1 texture: left red, right blue.
        let texture = Texture {
            width: 2,
            height: 1,
            rgba: vec![255, 0, 0, 255, 0, 0, 255, 255],
        };
        let region = full_region(&texture);
        let bounds = Bounds {
            x: 0,
            y: 0,
            width: 2,
            height: 1,
        };
        let mut image = Image {
            width: 2,
            height: 1,
            pixels: vec![0; 6],
        };
        draw_texture(&mut image, &bounds, &texture, &region, &sprite("t"));
        assert_eq!(image.pixels, vec![255, 0, 0, 0, 0, 255]);

        // flip_x mirrors horizontally: left becomes blue, right becomes red.
        let mut flip = sprite("t");
        flip.flip_x = true;
        let mut image = Image {
            width: 2,
            height: 1,
            pixels: vec![0; 6],
        };
        draw_texture(&mut image, &bounds, &texture, &region, &flip);
        assert_eq!(image.pixels, vec![0, 0, 255, 255, 0, 0]);

        // 1x2 texture: top green, bottom white; flip_y swaps rows.
        let texture = Texture {
            width: 1,
            height: 2,
            rgba: vec![0, 255, 0, 255, 255, 255, 255, 255],
        };
        let region = full_region(&texture);
        let bounds = Bounds {
            x: 0,
            y: 0,
            width: 1,
            height: 2,
        };
        let mut flip = sprite("t");
        flip.flip_y = true;
        let mut image = Image {
            width: 1,
            height: 2,
            pixels: vec![0; 6],
        };
        draw_texture(&mut image, &bounds, &texture, &region, &flip);
        assert_eq!(image.pixels, vec![255, 255, 255, 0, 255, 0]);
    }

    #[test]
    fn z_order_higher_drawn_on_top_and_id_tiebreak() {
        // Two 1x1 entities at the same screen position, overlapping.
        let toml = r#"[project]
name = "z-order"
format_version = 1

[simulation]
tick_rate = 60
seed = 1

[render]
width = 1
height = 1
background = [0, 0, 0]

[render.camera]
x = 0
y = 0
pixels_per_unit = 1

[[entities]]
id = 1
name = "under"
position = { x = 0, y = 0 }
appearance = { width = 1, height = 1, color = [10, 10, 10] }
sprite = { asset = "red", z = 0 }

[[entities]]
id = 2
name = "over"
position = { x = 0, y = 0 }
appearance = { width = 1, height = 1, color = [20, 20, 20] }
sprite = { asset = "blue", z = 5 }
"#;
        let project: Project = toml::from_str(toml).unwrap();
        let config = project.render.clone();
        let world = World::from_project(project);
        let mut textures = BTreeMap::new();
        textures.insert("red".to_string(), solid_texture([255, 0, 0, 255]));
        textures.insert("blue".to_string(), solid_texture([0, 0, 255, 255]));

        // Higher z (blue, z=5) must be drawn last / on top.
        let (image, _) = render_with_textures(&world, &config, &textures).unwrap();
        assert_eq!(image.pixels, vec![0, 0, 255]);

        // Equal z: tie-break by EntityId ascending (id 2 drawn last / on top).
        let toml_equal = toml.replace("z = 5", "z = 0");
        let project: Project = toml::from_str(&toml_equal).unwrap();
        let config = project.render.clone();
        let world = World::from_project(project);
        let (image, _) = render_with_textures(&world, &config, &textures).unwrap();
        assert_eq!(image.pixels, vec![0, 0, 255]);
    }

    #[test]
    fn resolve_animation_loop_and_once() {
        let mut base = sprite("t");
        base.animation = Some(AnimationConfig {
            frames: vec![
                AnimationFrame {
                    region: AtlasRegion {
                        x: 0,
                        y: 0,
                        width: 4,
                        height: 4,
                    },
                    duration_ticks: 2,
                },
                AnimationFrame {
                    region: AtlasRegion {
                        x: 4,
                        y: 0,
                        width: 4,
                        height: 4,
                    },
                    duration_ticks: 3,
                },
            ],
            mode: AnimationMode::Loop,
        });
        // Frame 0 covers ticks 0..2, frame 1 covers ticks 2..5, then loops (total 5).
        assert_eq!(resolve_animation_region(&base, 0).unwrap().x, 0);
        assert_eq!(resolve_animation_region(&base, 1).unwrap().x, 0);
        assert_eq!(resolve_animation_region(&base, 2).unwrap().x, 4);
        assert_eq!(resolve_animation_region(&base, 4).unwrap().x, 4);
        assert_eq!(resolve_animation_region(&base, 5).unwrap().x, 0); // wraps
        assert_eq!(resolve_animation_region(&base, 7).unwrap().x, 4);

        // Once mode clamps to the final frame after the cycle length.
        let mut once = base.clone();
        if let Some(anim) = once.animation.as_mut() {
            anim.mode = AnimationMode::Once;
        }
        assert_eq!(resolve_animation_region(&once, 0).unwrap().x, 0);
        assert_eq!(resolve_animation_region(&once, 4).unwrap().x, 4);
        assert_eq!(resolve_animation_region(&once, 100).unwrap().x, 4); // clamped
    }

    #[test]
    fn resolve_animation_none_uses_static_region() {
        let mut s = sprite("t");
        assert!(resolve_animation_region(&s, 3).is_none());
        s.region = Some(AtlasRegion {
            x: 2,
            y: 2,
            width: 8,
            height: 8,
        });
        assert_eq!(resolve_animation_region(&s, 9).unwrap().x, 2);
    }
}
