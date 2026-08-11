use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Result;

pub const PROJECT_FILE: &str = "aetherion.toml";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub project: Metadata,
    pub simulation: SimulationConfig,
    #[serde(default)]
    pub render: RenderConfig,
    #[serde(default)]
    pub entities: Vec<EntityConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    pub name: String,
    pub format_version: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SimulationConfig {
    pub tick_rate: u32,
    pub seed: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RenderConfig {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_background")]
    pub background: [u8; 3],
    #[serde(default)]
    pub camera: CameraConfig,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            background: default_background(),
            camera: CameraConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CameraConfig {
    #[serde(default)]
    pub x: i64,
    #[serde(default)]
    pub y: i64,
    #[serde(default = "default_pixels_per_unit")]
    pub pixels_per_unit: u32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            pixels_per_unit: default_pixels_per_unit(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EntityConfig {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub position: Position,
    #[serde(default)]
    pub velocity: Velocity,
    #[serde(default)]
    pub appearance: Appearance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sprite: Option<SpriteConfig>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteConfig {
    pub asset: String,
    #[serde(default)]
    pub region: Option<AtlasRegion>,
    #[serde(default)]
    pub pivot: Pivot,
    #[serde(default)]
    pub z: i32,
    #[serde(default = "visible")]
    pub visible: bool,
    #[serde(default = "white")]
    pub tint: [u8; 4],
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default)]
    pub animation: Option<AnimationConfig>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pivot {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationConfig {
    pub frames: Vec<AnimationFrame>,
    #[serde(default)]
    pub mode: AnimationMode,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationFrame {
    pub region: AtlasRegion,
    pub duration_ticks: u32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimationMode {
    #[default]
    Loop,
    Once,
}
const fn visible() -> bool {
    true
}
const fn white() -> [u8; 4] {
    [255, 255, 255, 255]
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    #[serde(default = "default_entity_size")]
    pub width: u32,
    #[serde(default = "default_entity_size")]
    pub height: u32,
    #[serde(default = "default_entity_color")]
    pub color: [u8; 3],
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            width: default_entity_size(),
            height: default_entity_size(),
            color: default_entity_color(),
        }
    }
}

const fn default_width() -> u32 {
    160
}
const fn default_height() -> u32 {
    120
}
const fn default_pixels_per_unit() -> u32 {
    4
}
const fn default_entity_size() -> u32 {
    2
}
const fn default_background() -> [u8; 3] {
    [16, 20, 28]
}
const fn default_entity_color() -> [u8; 3] {
    [80, 220, 120]
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Velocity {
    pub x: i64,
    pub y: i64,
}

impl Project {
    pub fn load(directory: &Path) -> Result<Self> {
        let path = directory.join(PROJECT_FILE);
        let text =
            fs::read_to_string(&path).map_err(|e| format!("lecture de {}: {e}", path.display()))?;
        let value: Self = toml::from_str(&text)
            .map_err(|e| format!("configuration invalide dans {}: {e}", path.display()))?;
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<()> {
        if self.project.format_version != 1 {
            return Err("format_version non supportée (attendu: 1)".into());
        }
        if self.project.name.trim().is_empty() {
            return Err("project.name ne peut pas être vide".into());
        }
        if self.simulation.tick_rate == 0 {
            return Err("simulation.tick_rate doit être supérieur à 0".into());
        }
        if self.render.width == 0 || self.render.height == 0 {
            return Err("render.width et render.height doivent être supérieurs à 0".into());
        }
        if self.render.width > 8192 || self.render.height > 8192 {
            return Err("dimensions de rendu limitées à 8192 pixels".into());
        }
        if !(1..=1024).contains(&self.render.camera.pixels_per_unit) {
            return Err("render.camera.pixels_per_unit doit être compris entre 1 et 1024".into());
        }
        if self
            .entities
            .iter()
            .any(|entity| entity.appearance.width == 0 || entity.appearance.height == 0)
        {
            return Err("la taille d'apparence des entités doit être supérieure à 0".into());
        }
        for entity in &self.entities {
            if let Some(sprite) = &entity.sprite {
                if sprite.asset.is_empty() || sprite.asset.len() > 128 {
                    return Err("sprite.asset invalide".into());
                }
                if let Some(region) = sprite.region
                    && (region.width == 0 || region.height == 0)
                {
                    return Err("région d'atlas vide".into());
                }
                if let Some(animation) = &sprite.animation
                    && (animation.frames.is_empty()
                        || animation.frames.len() > 4096
                        || animation.frames.iter().any(|frame| {
                            frame.duration_ticks == 0
                                || frame.region.width == 0
                                || frame.region.height == 0
                        }))
                {
                    return Err(
                        "animation invalide: 1..4096 frames non vides avec durée > 0".into(),
                    );
                }
            }
        }
        let mut ids: Vec<u64> = self.entities.iter().map(|e| e.id).collect();
        ids.sort_unstable();
        if ids.windows(2).any(|w| w[0] == w[1]) {
            return Err("les identifiants d'entités doivent être uniques".into());
        }
        Ok(())
    }

    pub fn example() -> &'static str {
        r#"[project]
name = "hello-aetherion"
format_version = 1

[simulation]
tick_rate = 60
seed = 1

[render]
width = 160
height = 120
background = [16, 20, 28]

[render.camera]
x = 0
y = 0
pixels_per_unit = 4

[[entities]]
id = 1
name = "player"
position = { x = 0, y = 0 }
velocity = { x = 1, y = 0 }

[[entities]]
id = 2
name = "beacon"
position = { x = 10, y = 5 }
velocity = { x = 0, y = 0 }
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_is_valid() {
        let project: Project = toml::from_str(Project::example()).unwrap();
        project.validate().unwrap();
        assert_eq!(project.entities.len(), 2);
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let mut project: Project = toml::from_str(Project::example()).unwrap();
        project.entities[1].id = 1;
        assert!(project.validate().unwrap_err().message.contains("uniques"));
    }
}
