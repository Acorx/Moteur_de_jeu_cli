use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::project::{
    CameraConfig, EntityConfig, Metadata, Project, RenderConfig, SimulationConfig,
};
use crate::simulation::World;

pub const SCENES_DIR: &str = "scenes";
pub const MAX_SCENE_BYTES: u64 = 1_048_576;
pub const MAX_SCENE_ENTITIES: usize = 10_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub schema: String,
    pub id: String,
    #[serde(default)]
    pub metadata: SceneMetadata,
    #[serde(default)]
    pub camera: CameraConfig,
    #[serde(default)]
    pub assets: Vec<String>,
    #[serde(default)]
    pub entities: Vec<EntityConfig>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SceneMetadata {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SceneSummary {
    pub id: String,
    pub path: String,
    pub title: String,
    pub entities: usize,
    pub assets: usize,
}

pub fn list(root: &Path) -> Result<Vec<SceneSummary>> {
    let directory = root.join(SCENES_DIR);
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(&directory)
        .map_err(|e| format!("scene_list: {}: {e}", directory.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let scene = load_file(&path)?;
            Ok(SceneSummary {
                id: scene.id,
                path: path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/"),
                title: scene.metadata.title,
                entities: scene.entities.len(),
                assets: scene.assets.len(),
            })
        })
        .collect()
}

pub fn load(root: &Path, id: &str) -> Result<Scene> {
    validate_id(id)?;
    let scene = load_file(&root.join(SCENES_DIR).join(format!("{id}.json")))?;
    if scene.id != id {
        return Err(format!("scene_id_mismatch: fichier {id}, contenu {}", scene.id).into());
    }
    Ok(scene)
}

fn load_file(path: &Path) -> Result<Scene> {
    let metadata =
        fs::metadata(path).map_err(|e| format!("scene_read: {}: {e}", path.display()))?;
    if metadata.len() > MAX_SCENE_BYTES {
        return Err("scene_too_large: plafond 1048576 octets".into());
    }
    let bytes = fs::read(path).map_err(|e| format!("scene_read: {}: {e}", path.display()))?;
    let scene: Scene = serde_json::from_slice(&bytes)
        .map_err(|e| format!("scene_invalid: {}: {e}", path.display()))?;
    validate(&scene)?;
    Ok(scene)
}

pub fn validate(scene: &Scene) -> Result<()> {
    if scene.schema != "aetherion.scene/v1" {
        return Err("scene_version: attendu aetherion.scene/v1".into());
    }
    validate_id(&scene.id)?;
    if scene.entities.len() > MAX_SCENE_ENTITIES {
        return Err("scene_entity_quota: plafond 10000".into());
    }
    let mut ids: Vec<u64> = scene.entities.iter().map(|e| e.id).collect();
    ids.sort_unstable();
    if ids.windows(2).any(|w| w[0] == w[1]) {
        return Err("scene_duplicate_entity: IDs uniques requis".into());
    }
    if !(1..=1024).contains(&scene.camera.pixels_per_unit) {
        return Err("scene_camera_invalid: pixels_per_unit 1..1024".into());
    }
    let mut assets = scene.assets.clone();
    assets.sort();
    if assets.windows(2).any(|w| w[0] == w[1]) {
        return Err("scene_duplicate_asset: références uniques requises".into());
    }
    Ok(())
}
pub fn declared_asset_ids(scene: &Scene) -> Vec<String> {
    let mut ids = scene.assets.clone();
    ids.extend(
        scene
            .entities
            .iter()
            .filter_map(|entity| entity.sprite.as_ref().map(|sprite| sprite.asset.clone())),
    );
    ids.sort();
    ids.dedup();
    ids
}

pub fn build_world(scene: &Scene, base: &Project) -> Result<World> {
    validate(scene)?;
    let project = Project {
        project: Metadata {
            name: scene.id.clone(),
            format_version: 1,
        },
        simulation: SimulationConfig {
            tick_rate: base.simulation.tick_rate,
            seed: base.simulation.seed,
        },
        render: RenderConfig {
            width: base.render.width,
            height: base.render.height,
            background: base.render.background,
            camera: scene.camera.clone(),
        },
        entities: scene.entities.clone(),
    };
    project.validate()?;
    Ok(World::from_project(project))
}

fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
    {
        return Err(format!("scene_id_invalid: {id}").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary_directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "aetherion-scene-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn load_happy_path_and_build_world() {
        let root = temporary_directory("happy");
        fs::create_dir_all(root.join(SCENES_DIR)).unwrap();
        fs::write(
            root.join(SCENES_DIR).join("demo.json"),
            r#"{
            "schema":"aetherion.scene/v1","id":"demo",
            "camera":{"x":1,"y":2,"pixels_per_unit":4},
            "entities":[{"id":9,"name":"one","position":{"x":0,"y":0},"velocity":{"x":0,"y":0}}]
        }"#,
        )
        .unwrap();
        let loaded = load(&root, "demo").unwrap();
        assert_eq!(loaded.id, "demo");
        let base: Project = toml::from_str(Project::example()).unwrap();
        assert_eq!(build_world(&loaded, &base).unwrap().entity_count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_json_and_missing_scene_are_errors() {
        let root = temporary_directory("errors");
        fs::create_dir_all(root.join(SCENES_DIR)).unwrap();
        fs::write(root.join(SCENES_DIR).join("bad.json"), "not json").unwrap();
        assert!(
            load(&root, "bad")
                .unwrap_err()
                .message
                .contains("scene_invalid")
        );
        assert!(
            load(&root, "missing")
                .unwrap_err()
                .message
                .contains("scene_read")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_version_is_rejected() {
        let s = Scene {
            schema: "x".into(),
            id: "ok".into(),
            metadata: SceneMetadata::default(),
            camera: CameraConfig::default(),
            assets: vec![],
            entities: vec![],
        };
        assert!(validate(&s).is_err());
    }
}
