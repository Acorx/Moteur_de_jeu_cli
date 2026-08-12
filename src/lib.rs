pub mod agent;
pub mod assets;
pub mod assets3d;
pub mod bundle;
pub mod capture;
pub mod certification;
pub mod diff;
pub mod display;
pub mod ecs;
pub mod error;
pub mod gltf3d;
pub mod gpu3d;
pub mod plugin;
#[cfg(feature = "plugin-runtime")]
pub mod plugin_audit;
pub mod plugin_lock;
#[cfg(feature = "plugin-runtime")]
pub mod plugin_run;
#[cfg(feature = "plugin-runtime")]
pub mod plugin_runtime;
pub mod project;
pub mod render;
pub mod render3d;
mod render3d_channels;
pub mod replay;
pub mod rng;
pub mod scenario;
pub mod scene;
pub mod scheduler;
pub mod schema;
pub mod script;
pub mod simulation;
pub mod telemetry;
pub mod visual_diff;
pub mod visual_diff3d;

use std::fs;
use std::path::PathBuf;

pub use error::AppError;
use project::{PROJECT_FILE, Project};
use simulation::World;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, PartialEq)]
pub enum Command {
    Help,
    Init {
        path: PathBuf,
        force: bool,
    },
    Doctor {
        path: PathBuf,
    },
    Inspect {
        path: PathBuf,
    },
    Run {
        path: PathBuf,
        ticks: u64,
        json: bool,
        telemetry: Option<PathBuf>,
    },
    Capture {
        path: PathBuf,
        ticks: u64,
        output: PathBuf,
        format: capture::ImageFormat,
        assets: Option<PathBuf>,
        scene: Option<String>,
        channels: capture::Channels,
    },
    CaptureMulti {
        path: PathBuf,
        ticks: u64,
        views: PathBuf,
        output_dir: PathBuf,
        assets: Option<PathBuf>,
        scene: Option<String>,
        channels: capture::Channels,
    },
    Capture3d {
        scene: PathBuf,
        output: PathBuf,
        width: u32,
        height: u32,
        ticks: u64,
        animation: Option<String>,
        assets: Option<PathBuf>,
        channels: capture::Channels,
    },
    GpuDemo {
        scene: PathBuf,
        width: u32,
        height: u32,
        assets: Option<PathBuf>,
    },
    GltfImport {
        input: PathBuf,
        output: PathBuf,
    },
    Asset3dImport {
        input: PathBuf,
        kind: assets3d::Asset3dType,
        output: PathBuf,
    },
    Play {
        path: PathBuf,
        max_ticks: Option<u64>,
    },
    ReplayCreate {
        path: PathBuf,
        ticks: u64,
        events: Option<PathBuf>,
        output: PathBuf,
        checkpoint_interval: u64,
    },
    ReplayRun {
        path: PathBuf,
        replay: PathBuf,
    },
    Diff {
        left: PathBuf,
        right: PathBuf,
    },
    VisualDiff {
        baseline: PathBuf,
        candidate: PathBuf,
        tolerances: visual_diff::Tolerances,
        report: Option<PathBuf>,
    },
    VisualDiff3d {
        baseline_manifest: PathBuf,
        candidate_manifest: PathBuf,
        tolerances: visual_diff3d::ChannelTolerances,
        report: PathBuf,
    },
    ScenarioRun {
        path: PathBuf,
        scenario: PathBuf,
        report: Option<PathBuf>,
        audit: Option<PathBuf>,
    },
    Agent {
        path: PathBuf,
        root: PathBuf,
        policy: Option<PathBuf>,
        audit: Option<PathBuf>,
    },
    SchemaList,
    SchemaShow {
        name: String,
    },
    SceneList {
        root: PathBuf,
    },
    SceneShow {
        root: PathBuf,
        id: String,
    },
    PluginValidate {
        manifest: PathBuf,
    },
    PluginInspect {
        manifest: PathBuf,
    },
    PluginList {
        root: PathBuf,
    },
    PluginResolve {
        dir: PathBuf,
        lockfile: PathBuf,
    },
    PluginLockCheck {
        dir: PathBuf,
        lockfile: PathBuf,
    },
    PluginRun {
        manifest: PathBuf,
        module: PathBuf,
        path: Option<PathBuf>,
        scene: Option<String>,
        assets: Option<PathBuf>,
        export: String,
        dry_run: bool,
        report: Option<PathBuf>,
    },
    PluginAudit {
        manifest: PathBuf,
        module: PathBuf,
        export: String,
        report: Option<PathBuf>,
    },
    ScriptRun {
        script: PathBuf,
        dry_run: bool,
        report: Option<PathBuf>,
    },
    Bundle {
        path: PathBuf,
        output: PathBuf,
    },
    BundleInspect {
        input: PathBuf,
    },
    CertifyM4 {
        report: PathBuf,
    },
}

impl Command {
    pub fn parse(args: &[String]) -> Result<Self> {
        let Some(name) = args.first().map(String::as_str) else {
            return Ok(Self::Help);
        };
        if matches!(name, "help" | "--help" | "-h") {
            return Ok(Self::Help);
        }
        if name == "schema" {
            return match args.get(1).map(String::as_str) {
                Some("list") => Ok(Self::SchemaList),
                Some("show") => Ok(Self::SchemaShow {
                    name: args.get(2).ok_or("schema show requiert un nom")?.clone(),
                }),
                _ => Err("usage: aetherion schema list|show <nom>".into()),
            };
        }
        if name == "plugin" {
            return match args.get(1).map(String::as_str) {
                Some("validate") => Ok(Self::PluginValidate {
                    manifest: PathBuf::from(
                        args.get(2).ok_or("plugin validate requiert un manifeste")?,
                    ),
                }),
                Some("inspect") => Ok(Self::PluginInspect {
                    manifest: PathBuf::from(
                        args.get(2).ok_or("plugin inspect requiert un manifeste")?,
                    ),
                }),
                Some("resolve") => {
                    let (dir, lockfile) = parse_plugin_lock_args(&args[2..])?;
                    Ok(Self::PluginResolve { dir, lockfile })
                }
                Some("lock-check") => {
                    let (dir, lockfile) = parse_plugin_lock_args(&args[2..])?;
                    Ok(Self::PluginLockCheck { dir, lockfile })
                }
                Some("run") => parse_plugin_run_args(&args[2..]),
                Some("audit") => parse_plugin_audit_args(&args[2..]),
                Some("list") => {
                    let root = match args.len() {
                        3 => PathBuf::from(&args[2]),
                        2 => PathBuf::from("plugins"),
                        _ => return Err("usage: aetherion plugin list [DOSSIER]".into()),
                    };
                    Ok(Self::PluginList { root })
                }
                _ => Err(
                    "usage: aetherion plugin validate|inspect MANIFESTE | plugin run|audit --manifest FILE --module FILE [options] | plugin list [DOSSIER]"
                        .into(),
                ),
            };
        }
        if name == "scene" {
            let action = args.get(1).map(String::as_str);
            let mut root = PathBuf::from(".");
            let mut i = if action == Some("show") { 3 } else { 2 };
            while i < args.len() {
                if args[i] != "--root" {
                    return Err(format!("option inconnue: {}", args[i]).into());
                }
                i += 1;
                root = PathBuf::from(args.get(i).ok_or("--root requiert une valeur")?);
                i += 1;
            }
            return match action {
                Some("list") => Ok(Self::SceneList { root }),
                Some("show") => Ok(Self::SceneShow {
                    root,
                    id: args.get(2).ok_or("scene show requiert un id")?.clone(),
                }),
                _ => Err(
                    "usage: aetherion scene list [--root PATH] | scene show <id> [--root PATH]"
                        .into(),
                ),
            };
        }
        let mut path = PathBuf::from(".");
        let mut ticks = None;
        let mut json = false;
        let mut force = false;
        let mut output = None;
        let mut events = None;
        let mut replay = None;
        let mut left = None;
        let mut right = None;
        let mut baseline = None;
        let mut candidate = None;
        let mut max_channel_delta = 0;
        let mut max_different_pixels = 0;
        let mut max_different_percent_milli = 0;
        let mut scenario_path = None;
        let mut report = None;
        let mut audit = None;
        let mut telemetry = None;
        let mut checkpoint_interval = None;
        let mut root = None;
        let mut policy = None;
        let mut format = capture::ImageFormat::Ppm;
        let mut views = None;
        let mut output_dir = None;
        let mut max_ticks = None;
        let mut assets = None;
        let mut scene = None;
        let mut channels = capture::Channels::default();
        let mut width = None;
        let mut height = None;
        let mut animation = None;
        let mut input = None;
        let mut asset3d_type = None;
        let mut baseline_manifest = None;
        let mut candidate_manifest = None;
        let mut color = visual_diff::Tolerances::default();
        let mut depth = visual_diff::Tolerances::default();
        let mut normals = visual_diff::Tolerances::default();
        let mut segmentation_max_different_pixels = 0;
        let mut dry_run = false;
        let mut script_path = None;
        let mut i = 1;
        while i < args.len() {
            let value = |index: usize, option: &str| -> Result<&String> {
                args.get(index)
                    .ok_or_else(|| format!("{option} requiert une valeur").into())
            };
            match args[i].as_str() {
                "--path" | "-p" => {
                    i += 1;
                    path = PathBuf::from(value(i, "--path")?);
                }
                "--ticks" | "-t" => {
                    i += 1;
                    ticks =
                        Some(value(i, "--ticks")?.parse::<u64>().map_err(|_| {
                            AppError::new("--ticks doit ÃƒÂªtre un entier positif")
                        })?);
                }
                "--json" => json = true,
                "--telemetry" => {
                    i += 1;
                    telemetry = Some(PathBuf::from(value(i, "--telemetry")?));
                }
                "--checkpoint-interval" => {
                    i += 1;
                    checkpoint_interval = Some(
                        value(i, "--checkpoint-interval")?
                            .parse::<u64>()
                            .map_err(|_| {
                                AppError::new(
                                    "--checkpoint-interval doit ÃƒÂªtre un entier positif",
                                )
                            })?,
                    );
                }
                "--force" => force = true,
                "--output" | "-o" => {
                    i += 1;
                    output = Some(PathBuf::from(value(i, "--output")?));
                }
                "--events" => {
                    i += 1;
                    events = Some(PathBuf::from(value(i, "--events")?));
                }
                "--replay" => {
                    i += 1;
                    replay = Some(PathBuf::from(value(i, "--replay")?));
                }
                "--left" => {
                    i += 1;
                    left = Some(PathBuf::from(value(i, "--left")?));
                }
                "--right" => {
                    i += 1;
                    right = Some(PathBuf::from(value(i, "--right")?));
                }
                "--baseline" => {
                    i += 1;
                    baseline = Some(PathBuf::from(value(i, "--baseline")?));
                }
                "--candidate" => {
                    i += 1;
                    candidate = Some(PathBuf::from(value(i, "--candidate")?));
                }
                "--max-channel-delta" => {
                    i += 1;
                    max_channel_delta =
                        value(i, "--max-channel-delta")?
                            .parse::<u32>()
                            .map_err(|_| {
                                AppError::new(
                                    "--max-channel-delta doit ÃƒÂªtre un entier non signÃƒÂ©",
                                )
                            })?;
                }
                "--max-different-pixels" => {
                    i += 1;
                    max_different_pixels = value(i, "--max-different-pixels")?
                        .parse::<u64>()
                        .map_err(|_| {
                            AppError::new(
                                "--max-different-pixels doit ÃƒÂªtre un entier non signÃƒÂ©",
                            )
                        })?;
                }
                "--max-different-percent-milli" => {
                    i += 1;
                    max_different_percent_milli = value(i, "--max-different-percent-milli")?
                        .parse::<u64>()
                        .map_err(|_| {
                            AppError::new(
                                "--max-different-percent-milli doit ÃƒÂªtre un entier non signÃƒÂ©",
                            )
                        })?;
                    if max_different_percent_milli > 100_000 {
                        return Err(
                            "--max-different-percent-milli doit ÃƒÂªtre compris entre 0 et 100000"
                                .into(),
                        );
                    }
                }
                "--scenario" => {
                    i += 1;
                    scenario_path = Some(PathBuf::from(value(i, "--scenario")?));
                }
                "--report" => {
                    i += 1;
                    report = Some(PathBuf::from(value(i, "--report")?));
                }
                "--audit" => {
                    i += 1;
                    audit = Some(PathBuf::from(value(i, "--audit")?));
                }
                "--root" => {
                    i += 1;
                    root = Some(PathBuf::from(value(i, "--root")?));
                }
                "--policy" => {
                    i += 1;
                    policy = Some(PathBuf::from(value(i, "--policy")?));
                }
                "--format" => {
                    i += 1;
                    format = capture::ImageFormat::parse(value(i, "--format")?)?;
                }
                "--views" => {
                    i += 1;
                    views = Some(PathBuf::from(value(i, "--views")?));
                }
                "--output-dir" => {
                    i += 1;
                    output_dir = Some(PathBuf::from(value(i, "--output-dir")?));
                }
                "--assets" => {
                    i += 1;
                    assets = Some(PathBuf::from(value(i, "--assets")?));
                }
                "--channels" => {
                    i += 1;
                    channels = capture::Channels::parse(value(i, "--channels")?)?;
                }
                "--scene" => {
                    i += 1;
                    scene = Some(value(i, "--scene")?.clone());
                }
                "--animation" => {
                    i += 1;
                    animation = Some(value(i, "--animation")?.clone());
                }
                "--input" => {
                    i += 1;
                    input = Some(PathBuf::from(value(i, "--input")?));
                }
                "--type" => {
                    i += 1;
                    asset3d_type = Some(match value(i, "--type")?.as_str() {
                        "mesh" => assets3d::Asset3dType::Mesh,
                        "material" => assets3d::Asset3dType::Material,
                        _ => return Err("--type doit valoir mesh ou material".into()),
                    });
                }
                "--width" => {
                    i += 1;
                    width =
                        Some(value(i, "--width")?.parse::<u32>().map_err(|_| {
                            AppError::new("--width doit ÃƒÂªtre un entier positif")
                        })?);
                }
                "--height" => {
                    i += 1;
                    height =
                        Some(value(i, "--height")?.parse::<u32>().map_err(|_| {
                            AppError::new("--height doit ÃƒÂªtre un entier positif")
                        })?);
                }
                "--baseline-manifest" => {
                    i += 1;
                    baseline_manifest = Some(PathBuf::from(value(i, "--baseline-manifest")?));
                }
                "--candidate-manifest" => {
                    i += 1;
                    candidate_manifest = Some(PathBuf::from(value(i, "--candidate-manifest")?));
                }
                "--color-max-channel-delta" => {
                    i += 1;
                    color.max_channel_delta = parse_u32_option(
                        value(i, "--color-max-channel-delta")?,
                        "--color-max-channel-delta",
                    )?;
                }
                "--color-max-different-pixels" => {
                    i += 1;
                    color.max_different_pixels = parse_u64_option(
                        value(i, "--color-max-different-pixels")?,
                        "--color-max-different-pixels",
                    )?;
                }
                "--color-max-different-percent-milli" => {
                    i += 1;
                    color.max_different_percent_milli = parse_percent_option(
                        value(i, "--color-max-different-percent-milli")?,
                        "--color-max-different-percent-milli",
                    )?;
                }
                "--depth-max-channel-delta" => {
                    i += 1;
                    depth.max_channel_delta = parse_u32_option(
                        value(i, "--depth-max-channel-delta")?,
                        "--depth-max-channel-delta",
                    )?;
                }
                "--depth-max-different-pixels" => {
                    i += 1;
                    depth.max_different_pixels = parse_u64_option(
                        value(i, "--depth-max-different-pixels")?,
                        "--depth-max-different-pixels",
                    )?;
                }
                "--depth-max-different-percent-milli" => {
                    i += 1;
                    depth.max_different_percent_milli = parse_percent_option(
                        value(i, "--depth-max-different-percent-milli")?,
                        "--depth-max-different-percent-milli",
                    )?;
                }
                "--normals-max-channel-delta" => {
                    i += 1;
                    normals.max_channel_delta = parse_u32_option(
                        value(i, "--normals-max-channel-delta")?,
                        "--normals-max-channel-delta",
                    )?;
                }
                "--normals-max-different-pixels" => {
                    i += 1;
                    normals.max_different_pixels = parse_u64_option(
                        value(i, "--normals-max-different-pixels")?,
                        "--normals-max-different-pixels",
                    )?;
                }
                "--normals-max-different-percent-milli" => {
                    i += 1;
                    normals.max_different_percent_milli = parse_percent_option(
                        value(i, "--normals-max-different-percent-milli")?,
                        "--normals-max-different-percent-milli",
                    )?;
                }
                "--segmentation-max-different-pixels" => {
                    i += 1;
                    segmentation_max_different_pixels = parse_u64_option(
                        value(i, "--segmentation-max-different-pixels")?,
                        "--segmentation-max-different-pixels",
                    )?;
                }
                "--script" => {
                    i += 1;
                    script_path = Some(PathBuf::from(value(i, "--script")?));
                }
                "--dry-run" => dry_run = true,
                "--max-ticks" => {
                    i += 1;
                    max_ticks = Some(value(i, "--max-ticks")?.parse::<u64>().map_err(|_| {
                        AppError::new("--max-ticks doit ÃƒÂªtre un entier positif")
                    })?);
                }
                other => return Err(format!("option inconnue: {other}").into()),
            }
            i += 1;
        }
        match name {
            "init" => Ok(Self::Init { path, force }),
            "doctor" => Ok(Self::Doctor { path }),
            "inspect" => Ok(Self::Inspect { path }),
            "run" => Ok(Self::Run {
                path,
                ticks: ticks.unwrap_or(10),
                json,
                telemetry,
            }),
            "capture" => Ok(Self::Capture {
                path,
                ticks: ticks.unwrap_or(0),
                output: output.unwrap_or_else(|| format!("capture.{}", format.extension()).into()),
                format,
                assets,
                scene,
                channels,
            }),
            "capture-multi" => Ok(Self::CaptureMulti {
                path,
                ticks: ticks.unwrap_or(0),
                views: views.ok_or("capture-multi requiert --views")?,
                output_dir: output_dir.ok_or("capture-multi requiert --output-dir")?,
                assets,
                scene,
                channels,
            }),
            "capture3d" => Ok(Self::Capture3d {
                scene: scene
                    .map(PathBuf::from)
                    .ok_or("capture3d requiert --scene")?,
                output: output.ok_or("capture3d requiert --output")?,
                width: width.unwrap_or(320),
                height: height.unwrap_or(240),
                ticks: ticks.unwrap_or(0),
                animation,
                assets,
                channels,
            }),
            "gpu-demo" => Ok(Self::GpuDemo {
                scene: scene
                    .map(PathBuf::from)
                    .ok_or("gpu-demo requiert --scene")?,
                width: width.unwrap_or(1280),
                height: height.unwrap_or(720),
                assets,
            }),
            "asset3d-import" => Ok(Self::Asset3dImport {
                input: input.ok_or("asset3d-import requiert --input")?,
                kind: asset3d_type.ok_or("asset3d-import requiert --type")?,
                output: output.ok_or("asset3d-import requiert --output")?,
            }),
            "gltf-import" => Ok(Self::GltfImport {
                input: input.ok_or("gltf-import requiert --input")?,
                output: output.ok_or("gltf-import requiert --output")?,
            }),
            "play" => Ok(Self::Play { path, max_ticks }),
            "replay-create" => Ok(Self::ReplayCreate {
                path,
                ticks: ticks.ok_or("replay-create requiert --ticks")?,
                events,
                output: output.ok_or("replay-create requiert --output")?,
                checkpoint_interval: checkpoint_interval.unwrap_or(1),
            }),
            "replay-run" => Ok(Self::ReplayRun {
                path,
                replay: replay.ok_or("replay-run requiert --replay")?,
            }),
            "diff" => Ok(Self::Diff {
                left: left.ok_or("diff requiert --left")?,
                right: right.ok_or("diff requiert --right")?,
            }),
            "visual-diff" => Ok(Self::VisualDiff {
                baseline: baseline.ok_or("visual-diff requiert --baseline")?,
                candidate: candidate.ok_or("visual-diff requiert --candidate")?,
                tolerances: visual_diff::Tolerances {
                    max_channel_delta,
                    max_different_pixels,
                    max_different_percent_milli,
                },
                report,
            }),
            "visual-diff3d" => Ok(Self::VisualDiff3d {
                baseline_manifest: baseline_manifest
                    .ok_or("visual-diff3d requiert --baseline-manifest")?,
                candidate_manifest: candidate_manifest
                    .ok_or("visual-diff3d requiert --candidate-manifest")?,
                tolerances: visual_diff3d::ChannelTolerances {
                    color,
                    depth,
                    normals,
                    segmentation: visual_diff::Tolerances {
                        max_different_pixels: segmentation_max_different_pixels,
                        max_different_percent_milli: if segmentation_max_different_pixels == 0 {
                            0
                        } else {
                            100_000
                        },
                        ..visual_diff::Tolerances::default()
                    },
                },
                report: report.ok_or("visual-diff3d requiert --report")?,
            }),
            "scenario-run" => Ok(Self::ScenarioRun {
                path,
                scenario: scenario_path.ok_or("scenario-run requiert --scenario")?,
                report,
                audit,
            }),
            "script-run" => Ok(Self::ScriptRun {
                script: script_path.ok_or("script-run requiert --script")?,
                dry_run,
                report,
            }),
            "bundle" => Ok(Self::Bundle {
                path,
                output: output.ok_or("bundle requiert --output")?,
            }),
            "bundle-inspect" => Ok(Self::BundleInspect {
                input: input.ok_or("bundle-inspect requiert --input")?,
            }),
            "certify-m4" => Ok(Self::CertifyM4 {
                report: report.ok_or("certify-m4 requiert --report")?,
            }),
            "agent" => Ok(Self::Agent {
                root: root.ok_or("agent requiert --root")?,
                path,
                policy,
                audit,
            }),
            other => Err(format!("commande inconnue: {other}").into()),
        }
    }
}

fn parse_plugin_run_args(args: &[String]) -> Result<Command> {
    let mut manifest = None;
    let mut module = None;
    let mut path = None;
    let mut scene = None;
    let mut assets = None;
    let mut export = plugin_runtime_entrypoint();
    let mut dry_run = false;
    let mut report = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--manifest" => {
                index += 1;
                manifest = Some(PathBuf::from(
                    args.get(index)
                        .ok_or("plugin run --manifest requiert une valeur")?,
                ));
            }
            "--module" => {
                index += 1;
                module = Some(PathBuf::from(
                    args.get(index)
                        .ok_or("plugin run --module requiert une valeur")?,
                ));
            }
            "--path" => {
                index += 1;
                path = Some(PathBuf::from(
                    args.get(index)
                        .ok_or("plugin run --path requiert une valeur")?,
                ));
            }
            "--scene" => {
                index += 1;
                scene = Some(
                    args.get(index)
                        .ok_or("plugin run --scene requiert une valeur")?
                        .clone(),
                );
            }
            "--assets" => {
                index += 1;
                assets = Some(PathBuf::from(
                    args.get(index)
                        .ok_or("plugin run --assets requiert une valeur")?,
                ));
            }
            "--export" => {
                index += 1;
                export = args
                    .get(index)
                    .ok_or("plugin run --export requiert une valeur")?
                    .clone();
            }
            "--dry-run" => dry_run = true,
            "--report" => {
                index += 1;
                report = Some(PathBuf::from(
                    args.get(index)
                        .ok_or("plugin run --report requiert une valeur")?,
                ));
            }
            other => return Err(format!("option inconnue pour plugin run: {other}").into()),
        }
        index += 1;
    }
    Ok(Command::PluginRun {
        manifest: manifest.ok_or("plugin run requiert --manifest")?,
        module: module.ok_or("plugin run requiert --module")?,
        path,
        scene,
        assets,
        export,
        dry_run,
        report,
    })
}

fn parse_plugin_audit_args(args: &[String]) -> Result<Command> {
    let mut manifest = None;
    let mut module = None;
    let mut export = plugin_runtime_entrypoint();
    let mut report = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--manifest" => {
                index += 1;
                manifest = Some(PathBuf::from(
                    args.get(index)
                        .ok_or("plugin audit --manifest requiert une valeur")?,
                ));
            }
            "--module" => {
                index += 1;
                module = Some(PathBuf::from(
                    args.get(index)
                        .ok_or("plugin audit --module requiert une valeur")?,
                ));
            }
            "--export" => {
                index += 1;
                export = args
                    .get(index)
                    .ok_or("plugin audit --export requiert une valeur")?
                    .clone();
            }
            "--report" => {
                index += 1;
                report = Some(PathBuf::from(
                    args.get(index)
                        .ok_or("plugin audit --report requiert une valeur")?,
                ));
            }
            other => return Err(format!("option inconnue pour plugin audit: {other}").into()),
        }
        index += 1;
    }
    Ok(Command::PluginAudit {
        manifest: manifest.ok_or("plugin audit requiert --manifest")?,
        module: module.ok_or("plugin audit requiert --module")?,
        export,
        report,
    })
}

fn plugin_runtime_entrypoint() -> String {
    "aetherion_main".into()
}

fn parse_plugin_lock_args(args: &[String]) -> Result<(PathBuf, PathBuf)> {
    let mut dir = None;
    let mut lockfile = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--dir" => {
                index += 1;
                dir = Some(PathBuf::from(
                    args.get(index).ok_or("--dir requiert une valeur")?,
                ));
            }
            "--lockfile" => {
                index += 1;
                lockfile = Some(PathBuf::from(
                    args.get(index).ok_or("--lockfile requiert une valeur")?,
                ));
            }
            other => return Err(format!("option inconnue: {other}").into()),
        }
        index += 1;
    }
    Ok((
        dir.ok_or("plugin resolve/lock-check requiert --dir")?,
        lockfile.unwrap_or_else(|| PathBuf::from("aetherion.plugin-lock.json")),
    ))
}

fn parse_u32_option(value: &str, option: &str) -> Result<u32> {
    value.parse().map_err(|_| {
        AppError::new(format!(
            "{option} doit Ã¢â€Å“Ã‚Â¬tre un entier non signÃ¢â€Å“Ã‚Â®"
        ))
    })
}

fn parse_u64_option(value: &str, option: &str) -> Result<u64> {
    value.parse().map_err(|_| {
        AppError::new(format!(
            "{option} doit Ã¢â€Å“Ã‚Â¬tre un entier non signÃ¢â€Å“Ã‚Â®"
        ))
    })
}

fn parse_percent_option(value: &str, option: &str) -> Result<u64> {
    let parsed = parse_u64_option(value, option)?;
    if parsed > 100_000 {
        return Err(format!("{option} doit Ã¢â€Å“Ã‚Â¬tre compris entre 0 et 100000").into());
    }
    Ok(parsed)
}

pub fn help() -> &'static str {
    "Aetherion 0.1.0 Ã¢â‚¬â€ moteur de jeu CLI headless et dÃƒÂ©terministe\n\nUSAGE:\n  aetherion <COMMANDE> [OPTIONS]\n\nCOMMANDES:\n  init           CrÃƒÂ©e un projet dÃƒÂ©claratif minimal\n  doctor         VÃƒÂ©rifie la configuration et l'environnement\n  inspect        Ãƒâ€°met le snapshot initial JSON\n  run            ExÃƒÂ©cute une simulation bornÃƒÂ©e\n  capture        Ãƒâ€°crit une image PPM/PNG et son manifeste JSON\n  capture-multi  Publie atomiquement plusieurs vues\n  capture3d      Rend une scÃƒÂ¨ne 3D headless en PPM\n  gpu-demo       Ouvre une scÃƒÂ¨ne 3D temps rÃƒÂ©el via wgpu\n  gltf-import    Convertit un fichier glTF/GLB en Scene3d canonique\n  play           Ouvre l'affichage Windows optionnel\n  replay-create  CrÃƒÂ©e un replay avec checkpoints configurables\n  replay-run     Rejoue et vÃƒÂ©rifie un replay v1 ou v2\n  diff           Compare deux snapshots ou manifestes JSON\n  visual-diff    Compare deux captures avec tolÃƒÂ©rances entiÃƒÂ¨res\n  scenario-run   ExÃƒÂ©cute un scÃƒÂ©nario agent-native bornÃƒÂ©\n  agent          Pilote un monde par JSONL sur stdin/stdout\n  schema         Liste ou affiche les schÃƒÂ©mas JSON publiÃƒÂ©s\n  scene          Liste ou affiche les scÃƒÂ¨nes JSON\n  plugin         Valide, inspecte, exécute ou liste les plugins\n  certify-m4     Certifie M4 et publie un rapport JSON dÃ¢â€Å“Ã‚Â®terministe\n  help           Affiche cette aide\n\nM4 prototype 3D:\n  capture3d --scene FILE --output FILE [--width N] [--height N] [--ticks N] [--animation ID] [--assets FILE] [--channels color,depth,normals,segmentation]\n\nM11 GPU:\n  gpu-demo --scene FILE [--assets FILE] [--width N] [--height N] (requiert --features render-gpu)\n\nM12 glTF:\n  gltf-import --input FILE --output FILE (requiert --features gltf-import)\n\nM4-D:\n  visual-diff --baseline FILE --candidate FILE [--max-channel-delta N] [--max-different-pixels N] [--max-different-percent-milli N] [--report FILE]\n\nM4-H:\n  visual-diff3d --baseline-manifest FILE --candidate-manifest FILE --report FILE [--color-max-channel-delta N] [--color-max-different-pixels N] [--color-max-different-percent-milli N] [options depth/normals Ã¢â€Å“Ã‚Â®quivalentes] [--segmentation-max-different-pixels N]\n\nM3:\n  capture --path DIR --ticks N --format ppm|png --output FILE [--assets FILE] [--scene ID] [--channels color,depth,normals,segmentation]\n  capture-multi --path DIR --views FILE --output-dir DIR [--ticks N] [--assets FILE] [--scene ID] [--channels color,depth,normals,segmentation]\n  scene list [--root PATH] | scene show ID [--root PATH]\n  play --path DIR [--max-ticks N] (requiert --features display)\n\nM2:\n  agent --path DIR --root DIR [--policy FILE] [--audit FILE]\n  schema list | schema show NOM\n\nCODES:\n  0 succÃƒÂ¨s, 1 diffÃƒÂ©rence/assertion ÃƒÂ©chouÃƒÂ©e, 2 usage/validation, 3 divergence/budget"
}

pub fn execute(command: Command) -> Result<Option<String>> {
    match command {
        Command::Help => Ok(Some(help().into())),
        Command::Init { path, force } => {
            fs::create_dir_all(&path)
                .map_err(|error| format!("crÃƒÂ©ation de {}: {error}", path.display()))?;
            let target = path.join(PROJECT_FILE);
            if target.exists() && !force {
                return Err(
                    format!("{} existe dÃƒÂ©jÃƒÂ  (utilisez --force)", target.display()).into(),
                );
            }
            fs::write(&target, Project::example())
                .map_err(|error| format!("ÃƒÂ©criture de {}: {error}", target.display()))?;
            Ok(Some(format!(
                "Projet Aetherion crÃƒÂ©ÃƒÂ©: {}",
                target.display()
            )))
        }
        Command::Doctor { path } => {
            let project = Project::load(&path)?;
            Ok(Some(format!(
                "OK: {} Ã¢â‚¬â€ {} entitÃƒÂ©(s), tick_rate={} Hz, seed={}",
                project.project.name,
                project.entities.len(),
                project.simulation.tick_rate,
                project.simulation.seed
            )))
        }
        Command::Inspect { path } => Ok(Some(
            World::from_project(Project::load(&path)?).snapshot_json()?,
        )),
        Command::Run {
            path,
            ticks,
            json,
            telemetry,
        } => {
            let mut world = World::from_project(Project::load(&path)?);
            world.run(ticks)?;
            if let Some(path) = telemetry {
                telemetry::save(&world.telemetry(), &path)?;
            }
            if json {
                Ok(Some(world.snapshot_json()?))
            } else {
                Ok(Some(format!(
                    "Simulation terminÃƒÂ©e: tick={}, entitÃƒÂ©s={}, checksum={}",
                    world.tick,
                    world.entity_count(),
                    world.checksum()
                )))
            }
        }
        Command::Capture {
            path,
            ticks,
            output,
            format,
            assets,
            scene,
            channels,
        } => {
            let project = Project::load(&path)?;
            let (mut world, render_config, asset_ids) =
                capture_world(&path, project, scene.as_deref())?;
            world.run(ticks)?;
            let textures = load_capture_textures(&path, assets.as_deref(), asset_ids)?;
            let manifest = capture::capture(
                &world,
                &render_config,
                &output,
                format,
                &textures,
                &channels,
            )?;
            Ok(Some(format!(
                "Capture ÃƒÂ©crite: {} (manifeste: {})",
                output.display(),
                manifest.display()
            )))
        }
        Command::CaptureMulti {
            path,
            ticks,
            views,
            output_dir,
            assets,
            scene,
            channels,
        } => {
            let project = Project::load(&path)?;
            let (mut world, render_config, asset_ids) =
                capture_world(&path, project, scene.as_deref())?;
            world.run(ticks)?;
            let textures = load_capture_textures(&path, assets.as_deref(), asset_ids)?;
            let manifest = capture::capture_multi(
                &world,
                &render_config,
                &capture::load_views(&views)?,
                &output_dir,
                &textures,
                &channels,
            )?;
            Ok(Some(format!(
                "Captures multi-vues ÃƒÂ©crites: {}",
                manifest.display()
            )))
        }
        Command::Capture3d {
            scene,
            output,
            width,
            height,
            ticks,
            animation,
            assets,
            channels,
        } => {
            let manifest = render3d::capture_with_assets(
                &scene,
                assets.as_deref(),
                &output,
                width,
                height,
                ticks,
                animation.as_deref(),
                &channels,
            )?;
            Ok(Some(format!(
                "Capture 3D ÃƒÂ©crite: {} (manifeste: {})",
                output.display(),
                manifest.display()
            )))
        }
        Command::GpuDemo {
            scene,
            width,
            height,
            assets,
        } => {
            gpu3d::run(&scene, assets.as_deref(), width, height)?;
            Ok(None)
        }
        Command::GltfImport { input, output } => {
            let summary = gltf3d::import(&input, &output)?;
            Ok(Some(serde_json::to_string_pretty(&summary).map_err(
                |error| format!("gltf_import_report_serialize: {error}"),
            )?))
        }
        Command::Asset3dImport {
            input,
            kind,
            output,
        } => {
            assets3d::import(&input, kind, &output)?;
            Ok(Some(format!(
                "Ressource 3D importÃƒÂ©e: {}",
                output.display()
            )))
        }
        Command::Play { path, max_ticks } => {
            display::play(Project::load(&path)?, max_ticks)?;
            Ok(None)
        }
        Command::ReplayCreate {
            path,
            ticks,
            events,
            output,
            checkpoint_interval,
        } => {
            let project = Project::load(&path)?;
            let events = events
                .as_deref()
                .map(replay::load_events)
                .transpose()?
                .unwrap_or_default();
            let replay = replay::create_with_interval(project, ticks, events, checkpoint_interval)?;
            replay::save(&replay, &output)?;
            Ok(Some(serde_json::json!({"schema":"aetherion.replay-created/v1","status":"created","path":output.to_string_lossy().replace('\\', "/"),"target_tick":ticks,"events":replay.events.len(),"checksums":replay.checksums.len()}).to_string()))
        }
        Command::ReplayRun {
            path,
            replay: replay_path,
        } => {
            let replay = replay::load(&replay_path)?;
            let report = replay::play(
                Project::load(&path)?,
                &replay,
                &replay_path.to_string_lossy(),
            )?;
            Ok(Some(serde_json::to_string_pretty(&report).map_err(
                |error| format!("sÃƒÂ©rialisation du rapport: {error}"),
            )?))
        }
        Command::Diff { left, right } => {
            Ok(Some(diff::outcome(diff::compare_files(&left, &right)?)?))
        }
        Command::VisualDiff {
            baseline,
            candidate,
            tolerances,
            report,
        } => Ok(Some(visual_diff::outcome(
            visual_diff::compare_files(&baseline, &candidate, tolerances)?,
            report.as_deref(),
        )?)),
        Command::VisualDiff3d {
            baseline_manifest,
            candidate_manifest,
            tolerances,
            report,
        } => Ok(Some(visual_diff3d::outcome(
            visual_diff3d::compare_files(&baseline_manifest, &candidate_manifest, tolerances)?,
            &report,
        )?)),
        Command::ScenarioRun {
            path,
            scenario: scenario_path,
            report,
            audit,
        } => {
            let result = scenario::run(&path, &scenario_path, report.as_deref(), audit.as_deref())?;
            Ok(Some(serde_json::to_string_pretty(&result).map_err(
                |error| format!("sÃƒÂ©rialisation du rapport: {error}"),
            )?))
        }
        Command::Agent {
            path,
            root,
            policy,
            audit,
        } => {
            agent::run(&path, &root, policy.as_deref(), audit.as_deref())?;
            Ok(None)
        }
        Command::SchemaList => Ok(Some(schema::list()?)),
        Command::SchemaShow { name } => Ok(Some(schema::show(&name)?)),
        Command::SceneList { root } => Ok(Some(
            serde_json::to_string_pretty(&scene::list(&root)?)
                .map_err(|error| format!("sÃƒÂ©rialisation des scÃƒÂ¨nes: {error}"))?,
        )),
        Command::SceneShow { root, id } => Ok(Some(
            serde_json::to_string_pretty(&scene::load(&root, &id)?)
                .map_err(|error| format!("sÃƒÂ©rialisation de la scÃƒÂ¨ne: {error}"))?,
        )),
        Command::PluginValidate { manifest } => Ok(Some(plugin::validation_report(&manifest)?)),
        Command::PluginInspect { manifest } => Ok(Some(plugin::inspect(&manifest)?)),
        Command::PluginList { root } => Ok(Some(plugin::catalog_json(&root)?)),
        Command::PluginResolve { dir, lockfile } => Ok(Some(
            serde_json::to_string_pretty(&plugin_lock::resolve(&dir, &lockfile)?)
                .map_err(|e| format!("plugin_lock_serialize: {e}"))?,
        )),
        Command::PluginLockCheck { dir, lockfile } => Ok(Some(
            serde_json::to_string_pretty(&plugin_lock::check(&dir, &lockfile)?)
                .map_err(|e| format!("plugin_lock_serialize: {e}"))?,
        )),
        Command::PluginRun {
            manifest,
            module,
            path,
            scene,
            assets,
            export,
            dry_run,
            report,
        } => {
            #[cfg(feature = "plugin-runtime")]
            {
                let value = plugin_run::run(plugin_run::RunOptions {
                    manifest,
                    module,
                    path,
                    scene,
                    assets,
                    export,
                    dry_run,
                    report,
                })?;
                Ok(Some(serde_json::to_string_pretty(&value).map_err(|e| {
                    format!("plugin_run_report_serialize: {e}")
                })?))
            }
            #[cfg(not(feature = "plugin-runtime"))]
            {
                let _ = (
                    manifest, module, path, scene, assets, export, dry_run, report,
                );
                Err("plugin_runtime_feature_disabled".into())
            }
        }
        Command::PluginAudit {
            manifest,
            module,
            export,
            report,
        } => {
            #[cfg(feature = "plugin-runtime")]
            {
                let value = plugin_audit::audit(plugin_audit::AuditOptions {
                    manifest,
                    module,
                    export,
                    report,
                })?;
                Ok(Some(serde_json::to_string_pretty(&value).map_err(|e| {
                    format!("plugin_audit_report_serialize: {e}")
                })?))
            }
            #[cfg(not(feature = "plugin-runtime"))]
            {
                let _ = (manifest, module, export, report);
                Err("plugin_runtime_feature_disabled".into())
            }
        }
        Command::ScriptRun {
            script: script_path,
            dry_run,
            report,
        } => Ok(Some(
            serde_json::to_string_pretty(&script::run(&script_path, dry_run, report.as_deref())?)
                .map_err(|e| format!("script_report_serialize: {e}"))?,
        )),
        Command::Bundle { path, output } => {
            bundle::create(&path, &output)?;
            Ok(Some(format!("Bundle â”œÂ®crit: {}", output.display())))
        }
        Command::BundleInspect { input } => Ok(Some(bundle::inspect(&input)?)),
        Command::CertifyM4 { report } => {
            let value = certification::certify(&report)?;
            Ok(Some(serde_json::to_string_pretty(&value).map_err(
                |error| format!("sÃ¢â€Å“Ã‚Â®rialisation de la certification M4: {error}"),
            )?))
        }
    }
}

fn project_asset_ids(project: &Project) -> Vec<String> {
    let mut ids: Vec<String> = project
        .entities
        .iter()
        .filter_map(|entity| entity.sprite.as_ref().map(|sprite| sprite.asset.clone()))
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

fn capture_world(
    path: &std::path::Path,
    project: Project,
    scene_id: Option<&str>,
) -> Result<(World, project::RenderConfig, Vec<String>)> {
    if let Some(id) = scene_id {
        let loaded = scene::load(path, id)?;
        let ids = scene::declared_asset_ids(&loaded);
        let render = project::RenderConfig {
            camera: loaded.camera.clone(),
            ..project.render.clone()
        };
        Ok((scene::build_world(&loaded, &project)?, render, ids))
    } else {
        let ids = project_asset_ids(&project);
        let render = project.render.clone();
        Ok((World::from_project(project), render, ids))
    }
}

fn load_capture_textures(
    project_root: &std::path::Path,
    manifest: Option<&std::path::Path>,
    ids: Vec<String>,
) -> Result<std::collections::BTreeMap<String, assets::Texture>> {
    let Some(manifest) = manifest else {
        return Ok(std::collections::BTreeMap::new());
    };
    let (root, requested) = if manifest.is_absolute() {
        let parent = manifest
            .parent()
            .ok_or("asset_manifest_invalid: parent absent")?;
        let file = manifest
            .file_name()
            .ok_or("asset_manifest_invalid: nom absent")?;
        (parent.to_path_buf(), PathBuf::from(file))
    } else {
        (project_root.to_path_buf(), manifest.to_path_buf())
    };
    let mut manager = assets::AssetManager::load(&root, Some(&requested))?;
    manager.prepare_concurrent(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bounded_run() {
        let args = ["run", "--ticks", "42", "--json"].map(str::to_string);
        assert_eq!(
            Command::parse(&args).unwrap(),
            Command::Run {
                path: PathBuf::from("."),
                ticks: 42,
                json: true,
                telemetry: None
            }
        );
    }

    #[test]
    fn rejects_unknown_command() {
        assert!(Command::parse(&["fly".into()]).is_err());
    }

    #[test]
    fn parses_gltf_import() {
        let args = [
            "gltf-import",
            "--input",
            "model.glb",
            "--output",
            "scene.json",
        ]
        .map(str::to_string);
        assert_eq!(
            Command::parse(&args).unwrap(),
            Command::GltfImport {
                input: PathBuf::from("model.glb"),
                output: PathBuf::from("scene.json"),
            }
        );
    }

    #[test]
    fn parses_gpu_demo_with_external_assets() {
        let args = [
            "gpu-demo",
            "--scene",
            "scene.json",
            "--assets",
            "assets.json",
            "--width",
            "1920",
            "--height",
            "1080",
        ]
        .map(str::to_string);
        assert_eq!(
            Command::parse(&args).unwrap(),
            Command::GpuDemo {
                scene: PathBuf::from("scene.json"),
                width: 1920,
                height: 1080,
                assets: Some(PathBuf::from("assets.json")),
            }
        );
    }

    #[test]
    fn parses_plugin_audit_with_report() {
        let args = [
            "plugin",
            "audit",
            "--manifest",
            "plugin.json",
            "--module",
            "plugin.wasm",
            "--report",
            "audit.json",
        ]
        .map(str::to_string);
        assert_eq!(
            Command::parse(&args).unwrap(),
            Command::PluginAudit {
                manifest: PathBuf::from("plugin.json"),
                module: PathBuf::from("plugin.wasm"),
                export: "aetherion_main".into(),
                report: Some(PathBuf::from("audit.json")),
            }
        );
    }

    #[test]
    fn parses_plugin_run_with_dry_run_and_report() {
        let args = [
            "plugin",
            "run",
            "--manifest",
            "plugin.json",
            "--module",
            "plugin.wasm",
            "--dry-run",
            "--report",
            "report.json",
            "--export",
            "entry",
        ]
        .map(str::to_string);
        assert_eq!(
            Command::parse(&args).unwrap(),
            Command::PluginRun {
                manifest: PathBuf::from("plugin.json"),
                module: PathBuf::from("plugin.wasm"),
                path: None,
                scene: None,
                assets: None,
                export: "entry".into(),
                dry_run: true,
                report: Some(PathBuf::from("report.json")),
            }
        );
    }
}
