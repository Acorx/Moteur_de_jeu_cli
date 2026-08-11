use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;
use crate::error::AppError;
use crate::visual_diff::{self, Tolerances, VisualDiffReport};

pub const REPORT_SCHEMA: &str = "aetherion.visual-diff3d/v1";
const CAPTURE_SCHEMA: &str = "aetherion.capture3d/v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChannelTolerances {
    pub color: Tolerances,
    pub depth: Tolerances,
    pub normals: Tolerances,
    pub segmentation: Tolerances,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    #[serde(default, rename = "scene_schema")]
    _scene_schema: Option<String>,
    width: u32,
    height: u32,
    #[serde(default, rename = "triangles")]
    _triangles: Option<u32>,
    #[serde(default, rename = "visible_pixels")]
    _visible_pixels: Option<u64>,
    #[serde(default, rename = "animation")]
    _animation: Option<String>,
    #[serde(default, rename = "tick")]
    _tick: Option<u64>,
    #[serde(default)]
    channels: Option<Vec<ManifestChannel>>,
    #[serde(default)]
    segmentation_mapping: Option<Vec<SegmentationMapping>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestChannel {
    name: String,
    file: String,
    encoding: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SegmentationMapping {
    id: u32,
    triangle_id: u32,
    source: String,
    rank: u32,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub baseline_manifest: String,
    pub candidate_manifest: String,
    pub width: u32,
    pub height: u32,
    pub passed: bool,
    pub channels: BTreeMap<String, VisualDiffReport>,
    pub missing_channels: Vec<String>,
    pub incompatible_channels: Vec<IncompatibleChannel>,
    pub segmentation_differences: Vec<SegmentationDifference>,
}

#[derive(Debug, Serialize)]
pub struct IncompatibleChannel {
    pub channel: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct SegmentationDifference {
    pub baseline_id: u32,
    pub candidate_id: u32,
    pub pixels: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_primitive: Option<Primitive>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_primitive: Option<Primitive>,
}

#[derive(Debug, Serialize)]
pub struct Primitive {
    pub triangle_id: u32,
    pub source: String,
    pub rank: u32,
}

pub fn compare_files(
    baseline_path: &Path,
    candidate_path: &Path,
    tolerances: ChannelTolerances,
) -> Result<Report> {
    let baseline = load_manifest(baseline_path)?;
    let candidate = load_manifest(candidate_path)?;
    if baseline.width != candidate.width || baseline.height != candidate.height {
        return Err(format!(
            "manifestes 3D incompatibles: dimensions {}x{} contre {}x{}",
            baseline.width, baseline.height, candidate.width, candidate.height
        )
        .into());
    }
    let baseline_channels = discover_channels(baseline_path, &baseline)?;
    let candidate_channels = discover_channels(candidate_path, &candidate)?;
    let names: BTreeSet<String> = baseline_channels
        .keys()
        .chain(candidate_channels.keys())
        .cloned()
        .collect();
    let mut channels = BTreeMap::new();
    let mut missing_channels = Vec::new();
    let mut incompatible_channels = Vec::new();
    for name in names {
        let (Some(left), Some(right)) =
            (baseline_channels.get(&name), candidate_channels.get(&name))
        else {
            missing_channels.push(name);
            continue;
        };
        if left.encoding != right.encoding {
            incompatible_channels.push(IncompatibleChannel {
                channel: name,
                reason: format!("encodages {} contre {}", left.encoding, right.encoding),
            });
            continue;
        }
        let tolerance = match name.as_str() {
            "color" => tolerances.color,
            "depth" => tolerances.depth,
            "normals" => tolerances.normals,
            "segmentation" => tolerances.segmentation,
            _ => continue,
        };
        match visual_diff::compare_files(&left.path, &right.path, tolerance) {
            Ok(report) => {
                channels.insert(name, report);
            }
            Err(error) => incompatible_channels.push(IncompatibleChannel {
                channel: name,
                reason: error.message,
            }),
        }
    }
    if !missing_channels.is_empty() || !incompatible_channels.is_empty() {
        return Err(format!(
            "manifestes 3D incompatibles: canaux manquants={:?}, incompatibles={}",
            missing_channels,
            incompatible_channels.len()
        )
        .into());
    }
    let segmentation_differences = match (
        baseline_channels.get("segmentation"),
        candidate_channels.get("segmentation"),
    ) {
        (Some(left), Some(right)) => segmentation_summary(
            &left.path,
            &right.path,
            baseline.segmentation_mapping.as_deref().unwrap_or_default(),
            candidate
                .segmentation_mapping
                .as_deref()
                .unwrap_or_default(),
        )?,
        _ => Vec::new(),
    };
    let passed = channels.values().all(|report| report.passed);
    Ok(Report {
        schema: REPORT_SCHEMA,
        baseline_manifest: normalized(baseline_path),
        candidate_manifest: normalized(candidate_path),
        width: baseline.width,
        height: baseline.height,
        passed,
        channels,
        missing_channels,
        incompatible_channels,
        segmentation_differences,
    })
}

pub fn outcome(report: Report, report_path: &Path) -> Result<String> {
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("sérialisation du visual diff 3D: {error}"))?;
    write_atomic(report_path, format!("{json}\n").as_bytes())?;
    if report.passed {
        Ok(json)
    } else {
        Err(AppError::outcome(
            "la comparaison visuelle 3D dépasse les tolérances",
            1,
            json,
        ))
    }
}

struct ChannelPath {
    path: PathBuf,
    encoding: String,
}

fn load_manifest(path: &Path) -> Result<Manifest> {
    let bytes = fs::read(path)
        .map_err(|error| format!("lecture du manifeste {}: {error}", path.display()))?;
    let manifest: Manifest = serde_json::from_slice(&bytes)
        .map_err(|error| format!("manifeste capture3d invalide: {error}"))?;
    if manifest.schema != CAPTURE_SCHEMA {
        return Err(format!("schéma capture3d invalide: attendu {CAPTURE_SCHEMA}").into());
    }
    if manifest.width == 0 || manifest.height == 0 {
        return Err("dimensions capture3d invalides".into());
    }
    Ok(manifest)
}

fn discover_channels(path: &Path, manifest: &Manifest) -> Result<BTreeMap<String, ChannelPath>> {
    let mut result = BTreeMap::new();
    let color = color_path(path)?;
    result.insert(
        "color".into(),
        ChannelPath {
            encoding: encoding_from_path(&color)?.into(),
            path: color,
        },
    );
    for channel in manifest.channels.as_deref().unwrap_or_default() {
        if !matches!(channel.name.as_str(), "depth" | "normals" | "segmentation") {
            return Err(format!("canal capture3d inconnu: {}", channel.name).into());
        }
        if result.contains_key(&channel.name) {
            return Err(format!("canal capture3d dupliqué: {}", channel.name).into());
        }
        let file = resolve_manifest_file(path, &channel.file)?;
        result.insert(
            channel.name.clone(),
            ChannelPath {
                path: file,
                encoding: channel.encoding.clone(),
            },
        );
    }
    Ok(result)
}

fn color_path(manifest: &Path) -> Result<PathBuf> {
    let text = manifest.to_string_lossy();
    let Some(value) = text.strip_suffix(".json") else {
        return Err("le manifeste capture3d doit porter le suffixe .json".into());
    };
    Ok(PathBuf::from(value))
}

fn resolve_manifest_file(manifest: &Path, file: &str) -> Result<PathBuf> {
    let declared = Path::new(file);
    if declared.is_absolute() {
        return Ok(declared.to_path_buf());
    }
    if declared.exists() {
        return Ok(declared.to_path_buf());
    }
    Ok(manifest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(declared.file_name().ok_or("chemin de canal invalide")?))
}

fn encoding_from_path(path: &Path) -> Result<&'static str> {
    match path.extension().and_then(|value| value.to_str()) {
        Some("ppm") => Ok("ppm-p6-rgb8"),
        Some("png") => Ok("png-rgb8"),
        _ => Err(format!("format couleur capture3d inconnu: {}", path.display()).into()),
    }
}

fn segmentation_summary(
    baseline_path: &Path,
    candidate_path: &Path,
    baseline_mapping: &[SegmentationMapping],
    candidate_mapping: &[SegmentationMapping],
) -> Result<Vec<SegmentationDifference>> {
    let (baseline_width, baseline_height, baseline_ids) =
        visual_diff::decode_rgb_ids(baseline_path)?;
    let (candidate_width, candidate_height, candidate_ids) =
        visual_diff::decode_rgb_ids(candidate_path)?;
    if baseline_width != candidate_width || baseline_height != candidate_height {
        return Err("dimensions de segmentation incompatibles".into());
    }
    let baseline: BTreeMap<u32, &SegmentationMapping> = baseline_mapping
        .iter()
        .map(|item| (item.id, item))
        .collect();
    let candidate: BTreeMap<u32, &SegmentationMapping> = candidate_mapping
        .iter()
        .map(|item| (item.id, item))
        .collect();
    let mut counts = BTreeMap::<(u32, u32), u64>::new();
    for (baseline_id, candidate_id) in baseline_ids.into_iter().zip(candidate_ids) {
        if baseline_id != candidate_id {
            *counts.entry((baseline_id, candidate_id)).or_default() += 1;
        }
    }
    Ok(counts
        .into_iter()
        .map(
            |((baseline_id, candidate_id), pixels)| SegmentationDifference {
                baseline_id,
                candidate_id,
                pixels,
                baseline_primitive: baseline.get(&baseline_id).map(|item| primitive(item)),
                candidate_primitive: candidate.get(&candidate_id).map(|item| primitive(item)),
            },
        )
        .collect())
}

fn primitive(item: &SegmentationMapping) -> Primitive {
    Primitive {
        triangle_id: item.triangle_id,
        source: item.source.clone(),
        rank: item.rank,
    }
}

fn normalized(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("création de {}: {error}", parent.display()))?;
    let name = path
        .file_name()
        .ok_or("nom de rapport invalide")?
        .to_string_lossy();
    let temporary = parent.join(format!(".{name}.aetherion-tmp-{}", std::process::id()));
    fs::write(&temporary, bytes)
        .map_err(|error| format!("écriture temporaire de {}: {error}", path.display()))?;
    if path.exists() {
        fs::remove_file(path)
            .map_err(|error| format!("remplacement de {}: {error}", path.display()))?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("publication atomique de {}: {error}", path.display()).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn directory() -> PathBuf {
        std::env::temp_dir().join(format!(
            "aetherion-visual-diff3d-unit-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn ppm(path: &Path, pixels: &[[u8; 3]]) {
        let mut bytes = format!("P6\n{} 1\n255\n", pixels.len()).into_bytes();
        for pixel in pixels {
            bytes.extend_from_slice(pixel);
        }
        fs::write(path, bytes).unwrap();
    }

    fn pgm(path: &Path, pixels: &[u16]) {
        let mut bytes = format!("P5\n{} 1\n65535\n", pixels.len()).into_bytes();
        for pixel in pixels {
            bytes.extend_from_slice(&pixel.to_be_bytes());
        }
        fs::write(path, bytes).unwrap();
    }

    fn manifest(path: &Path, stem: &str, mapping: serde_json::Value) {
        let value = serde_json::json!({
            "schema": CAPTURE_SCHEMA,
            "width": 2,
            "height": 1,
            "channels": [
                {"name":"depth", "file":format!("{stem}.depth.pgm"), "encoding":"pgm-p5-u16be"},
                {"name":"normals", "file":format!("{stem}.normals.ppm"), "encoding":"ppm-p6-rgb8"},
                {"name":"segmentation", "file":format!("{stem}.segmentation.ppm"), "encoding":"ppm-p6-rgb8"}
            ],
            "segmentation_mapping": mapping
        });
        fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    }

    fn fixture(
        root: &Path,
        stem: &str,
        color: [[u8; 3]; 2],
        depth: [u16; 2],
        normals: [[u8; 3]; 2],
        segmentation: [[u8; 3]; 2],
        mapping: serde_json::Value,
    ) -> PathBuf {
        ppm(&root.join(format!("{stem}.ppm")), &color);
        pgm(&root.join(format!("{stem}.depth.pgm")), &depth);
        ppm(&root.join(format!("{stem}.normals.ppm")), &normals);
        ppm(
            &root.join(format!("{stem}.segmentation.ppm")),
            &segmentation,
        );
        let path = root.join(format!("{stem}.ppm.json"));
        manifest(&path, stem, mapping);
        path
    }

    #[test]
    fn identical_channels_pass_and_report_is_deterministic() {
        let root = directory();
        fs::create_dir_all(&root).unwrap();
        let mapping = serde_json::json!([{"id":1,"triangle_id":0,"source":"mesh","rank":0}]);
        let baseline = fixture(
            &root,
            "baseline",
            [[1, 2, 3], [4, 5, 6]],
            [7, 8],
            [[128, 128, 255]; 2],
            [[0, 0, 1]; 2],
            mapping.clone(),
        );
        let candidate = fixture(
            &root,
            "candidate",
            [[1, 2, 3], [4, 5, 6]],
            [7, 8],
            [[128, 128, 255]; 2],
            [[0, 0, 1]; 2],
            mapping,
        );
        let first = serde_json::to_vec(
            &compare_files(&baseline, &candidate, ChannelTolerances::default()).unwrap(),
        )
        .unwrap();
        let second = serde_json::to_vec(
            &compare_files(&baseline, &candidate, ChannelTolerances::default()).unwrap(),
        )
        .unwrap();
        assert_eq!(first, second);
        assert!(
            serde_json::from_slice::<serde_json::Value>(&first).unwrap()["passed"]
                .as_bool()
                .unwrap()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn channel_tolerances_and_segmentation_mapping_are_reported() {
        let root = directory();
        fs::create_dir_all(&root).unwrap();
        let baseline = fixture(
            &root,
            "baseline",
            [[1, 2, 3], [4, 5, 6]],
            [7, 8],
            [[128, 128, 255]; 2],
            [[0, 0, 1], [0, 0, 1]],
            serde_json::json!([{"id":1,"triangle_id":0,"source":"left","rank":0}]),
        );
        let candidate = fixture(
            &root,
            "candidate",
            [[2, 2, 3], [4, 5, 6]],
            [9, 8],
            [[130, 128, 255], [128, 128, 255]],
            [[0, 0, 2], [0, 0, 1]],
            serde_json::json!([{"id":2,"triangle_id":1,"source":"right","rank":1}]),
        );
        let strict = compare_files(&baseline, &candidate, ChannelTolerances::default()).unwrap();
        assert!(!strict.passed);
        assert_eq!(strict.segmentation_differences[0].baseline_id, 1);
        assert_eq!(strict.segmentation_differences[0].candidate_id, 2);
        assert_eq!(
            strict.segmentation_differences[0]
                .baseline_primitive
                .as_ref()
                .unwrap()
                .source,
            "left"
        );
        assert_eq!(
            strict.segmentation_differences[0]
                .candidate_primitive
                .as_ref()
                .unwrap()
                .source,
            "right"
        );
        let tolerant = compare_files(
            &baseline,
            &candidate,
            ChannelTolerances {
                color: Tolerances {
                    max_channel_delta: 1,
                    ..Tolerances::default()
                },
                depth: Tolerances {
                    max_channel_delta: 2,
                    ..Tolerances::default()
                },
                normals: Tolerances {
                    max_channel_delta: 2,
                    ..Tolerances::default()
                },
                segmentation: Tolerances {
                    max_different_pixels: 1,
                    max_different_percent_milli: 100_000,
                    ..Tolerances::default()
                },
            },
        )
        .unwrap();
        assert!(tolerant.passed);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_or_incompatible_channels_are_validation_errors() {
        let root = directory();
        fs::create_dir_all(&root).unwrap();
        let mapping = serde_json::json!([]);
        let baseline = fixture(
            &root,
            "baseline",
            [[0, 0, 0]; 2],
            [0; 2],
            [[0, 0, 0]; 2],
            [[0, 0, 0]; 2],
            mapping.clone(),
        );
        let candidate = fixture(
            &root,
            "candidate",
            [[0, 0, 0]; 2],
            [0; 2],
            [[0, 0, 0]; 2],
            [[0, 0, 0]; 2],
            mapping,
        );
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&candidate).unwrap()).unwrap();
        value["channels"].as_array_mut().unwrap().pop();
        fs::write(&candidate, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(compare_files(&baseline, &candidate, ChannelTolerances::default()).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn report_replacement_leaves_no_temporary_file() {
        let root = directory();
        fs::create_dir_all(&root).unwrap();
        let report = root.join("report.json");
        write_atomic(&report, b"first").unwrap();
        write_atomic(&report, b"second").unwrap();
        assert_eq!(fs::read(&report).unwrap(), b"second");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}
