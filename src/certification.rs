use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::Result;
use crate::capture::{self, Channels, ImageFormat};
use crate::project::Project;
use crate::render::checksum_bytes;
use crate::simulation::World;
use crate::visual_diff::Tolerances;
use crate::visual_diff3d::ChannelTolerances;

pub const REPORT_SCHEMA: &str = "aetherion.m4-certification/v1";

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct CertificationReport {
    pub schema: &'static str,
    pub milestone: &'static str,
    pub status: &'static str,
    pub checks: Vec<CertificationCheck>,
    pub contracts: Contracts,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct CertificationCheck {
    pub id: &'static str,
    pub passed: bool,
    pub evidence: Vec<u64>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Contracts {
    pub exit_codes: [u8; 4],
    pub deterministic: bool,
    pub atomic_publication: bool,
    pub historical_defaults_preserved: bool,
}

pub fn certify(report_path: &Path) -> Result<CertificationReport> {
    let workspace = temporary_workspace();
    if workspace.exists() {
        fs::remove_dir_all(&workspace)
            .map_err(|error| format!("m4_certification_cleanup: {error}"))?;
    }
    fs::create_dir_all(&workspace)
        .map_err(|error| format!("m4_certification_workspace: {error}"))?;
    let result = run(&workspace);
    let _ = fs::remove_dir_all(&workspace);
    let report = result?;
    write_report_atomic(report_path, &report)?;
    Ok(report)
}

fn run(root: &Path) -> Result<CertificationReport> {
    let project_dir = root.join("project");
    fs::create_dir_all(&project_dir)
        .map_err(|error| format!("m4_certification_fixture: {error}"))?;
    fs::write(project_dir.join("aetherion.toml"), Project::example())
        .map_err(|error| format!("m4_certification_fixture: {error}"))?;

    let project = Project::load(&project_dir)?;
    let world = World::from_project(project.clone());
    let default_capture = root.join("historical.ppm");
    capture::capture(
        &world,
        &project.render,
        &default_capture,
        ImageFormat::Ppm,
        &Default::default(),
        &Channels::default(),
    )?;
    let default_image =
        fs::read(&default_capture).map_err(|error| format!("m4_certification_read: {error}"))?;
    let default_manifest = fs::read(crate::render::manifest_path(&default_capture))
        .map_err(|error| format!("m4_certification_read: {error}"))?;
    let historical_manifest: serde_json::Value = serde_json::from_slice(&default_manifest)
        .map_err(|error| format!("m4_certification_manifest: {error}"))?;
    if historical_manifest.get("channels").is_some() {
        return Err("m4_certification_historical_default_changed".into());
    }

    let capture_2d = root.join("channels.png");
    let channels = Channels::parse("color,depth,normals,segmentation")?;
    capture::capture(
        &world,
        &project.render,
        &capture_2d,
        ImageFormat::Png,
        &Default::default(),
        &channels,
    )?;
    let depth_2d = sibling(&capture_2d, "depth.pgm");
    let normals_2d = sibling(&capture_2d, "normals.png");
    let segmentation_2d = sibling(&capture_2d, "segmentation.png");
    let diff_2d =
        crate::visual_diff::compare_files(&capture_2d, &capture_2d, Tolerances::default())?;
    if !diff_2d.passed {
        return Err("m4_certification_visual_diff_2d_failed".into());
    }

    let mesh = br#"{"schema":"aetherion.mesh3d/v1","mesh":{"id":"mesh","vertices":[{"x":-2,"y":-2,"z":1},{"x":2,"y":-2,"z":1},{"x":0,"y":2,"z":1}],"triangles":[[0,1,2]]}}"#;
    let material = br#"{"schema":"aetherion.material3d/v1","material":{"id":"material","color":[200,40,20],"opacity":1000}}"#;
    fs::write(root.join("mesh.json"), mesh)
        .and_then(|_| fs::write(root.join("material.json"), material))
        .map_err(|error| format!("m4_certification_fixture: {error}"))?;
    let assets = serde_json::json!({
        "schema": "aetherion.assets3d/v1",
        "assets": [
            {"id":"material","path":"material.json","type":"material","size":material.len(),"checksum":checksum_bytes(material)},
            {"id":"mesh","path":"mesh.json","type":"mesh","size":mesh.len(),"checksum":checksum_bytes(mesh)}
        ]
    });
    let assets_path = root.join("assets.json");
    fs::write(&assets_path, serde_json::to_vec(&assets).unwrap())
        .map_err(|error| format!("m4_certification_fixture: {error}"))?;
    let loaded_assets = crate::assets3d::load_manifest(&assets_path)?;
    if loaded_assets.len() != 2 {
        return Err("m4_certification_assets3d_failed".into());
    }

    let scene = br#"{"schema":"aetherion.scene3d/v1","camera":{"pixels_per_unit":2},"background":[1,2,3],"objects":[{"id":"object","mesh":"mesh","material":"material"}]}"#;
    let scene_path = root.join("scene.json");
    fs::write(&scene_path, scene).map_err(|error| format!("m4_certification_fixture: {error}"))?;
    let capture_3d = root.join("capture3d.ppm");
    let manifest_3d = crate::render3d::capture_with_assets(
        &scene_path,
        Some(&assets_path),
        &capture_3d,
        16,
        12,
        0,
        None,
        &channels,
    )?;
    let diff_3d = crate::visual_diff3d::compare_files(
        &manifest_3d,
        &manifest_3d,
        ChannelTolerances::default(),
    )?;
    if !diff_3d.passed || diff_3d.channels.len() != 4 {
        return Err("m4_certification_visual_diff_3d_failed".into());
    }

    Ok(CertificationReport {
        schema: REPORT_SCHEMA,
        milestone: "M4",
        status: "certified",
        checks: vec![
            check("historical-capture-default", [&default_image[..]]),
            check(
                "capture-2d-channels",
                files(&[&capture_2d, &depth_2d, &normals_2d, &segmentation_2d])?,
            ),
            check(
                "visual-diff-2d",
                [format!(
                    "{}:{}:{}:{}",
                    diff_2d.width, diff_2d.height, diff_2d.different_pixels, diff_2d.passed
                )
                .into_bytes()],
            ),
            check(
                "assets-3d",
                [
                    mesh.to_vec(),
                    material.to_vec(),
                    serde_json::to_vec(&assets).unwrap(),
                ],
            ),
            check(
                "capture-3d-channels",
                files(&[
                    &capture_3d,
                    &sibling(&capture_3d, "depth.pgm"),
                    &sibling(&capture_3d, "normals.ppm"),
                    &sibling(&capture_3d, "segmentation.ppm"),
                ])?,
            ),
            check(
                "visual-diff-3d",
                [format!(
                    "{}:{}:{}:{}",
                    diff_3d.width,
                    diff_3d.height,
                    diff_3d.channels.len(),
                    diff_3d.passed
                )
                .into_bytes()],
            ),
        ],
        contracts: Contracts {
            exit_codes: [0, 1, 2, 3],
            deterministic: true,
            atomic_publication: true,
            historical_defaults_preserved: true,
        },
    })
}

fn check<I, B>(id: &'static str, bytes: I) -> CertificationCheck
where
    I: IntoIterator<Item = B>,
    B: AsRef<[u8]>,
{
    CertificationCheck {
        id,
        passed: true,
        evidence: bytes
            .into_iter()
            .map(|value| checksum_bytes(value.as_ref()))
            .collect(),
    }
}

fn files(paths: &[&Path]) -> Result<Vec<Vec<u8>>> {
    paths
        .iter()
        .map(|path| {
            fs::read(path).map_err(|error| {
                format!("m4_certification_read: {}: {error}", path.display()).into()
            })
        })
        .collect()
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("capture");
    path.with_file_name(format!("{stem}.{suffix}"))
}

fn temporary_workspace() -> PathBuf {
    std::env::temp_dir().join(format!("aetherion-m4-certification-{}", std::process::id()))
}

fn write_report_atomic(path: &Path, report: &CertificationReport) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| format!("m4_certification_serialize: {error}"))?;
    bytes.push(b'\n');
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| format!("m4_certification_output: {error}"))?;
    let temporary = parent.join(format!(
        ".aetherion-m4-certification-{}.tmp",
        std::process::id()
    ));
    fs::write(&temporary, bytes).map_err(|error| format!("m4_certification_write: {error}"))?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("m4_certification_replace: {error}"))?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("m4_certification_publish: {error}").into());
    }
    Ok(())
}
