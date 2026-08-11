pub mod agent;
pub mod assets;
pub mod assets3d;
pub mod capture;
pub mod diff;
pub mod display;
pub mod ecs;
pub mod error;
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
                    ticks = Some(
                        value(i, "--ticks")?
                            .parse::<u64>()
                            .map_err(|_| AppError::new("--ticks doit être un entier positif"))?,
                    );
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
                                AppError::new("--checkpoint-interval doit être un entier positif")
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
                                AppError::new("--max-channel-delta doit être un entier non signé")
                            })?;
                }
                "--max-different-pixels" => {
                    i += 1;
                    max_different_pixels = value(i, "--max-different-pixels")?
                        .parse::<u64>()
                        .map_err(|_| {
                            AppError::new("--max-different-pixels doit être un entier non signé")
                        })?;
                }
                "--max-different-percent-milli" => {
                    i += 1;
                    max_different_percent_milli = value(i, "--max-different-percent-milli")?
                        .parse::<u64>()
                        .map_err(|_| {
                            AppError::new(
                                "--max-different-percent-milli doit être un entier non signé",
                            )
                        })?;
                    if max_different_percent_milli > 100_000 {
                        return Err(
                            "--max-different-percent-milli doit être compris entre 0 et 100000"
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
                    width = Some(
                        value(i, "--width")?
                            .parse::<u32>()
                            .map_err(|_| AppError::new("--width doit être un entier positif"))?,
                    );
                }
                "--height" => {
                    i += 1;
                    height = Some(
                        value(i, "--height")?
                            .parse::<u32>()
                            .map_err(|_| AppError::new("--height doit être un entier positif"))?,
                    );
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
                "--max-ticks" => {
                    i += 1;
                    max_ticks =
                        Some(value(i, "--max-ticks")?.parse::<u64>().map_err(|_| {
                            AppError::new("--max-ticks doit être un entier positif")
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
            "asset3d-import" => Ok(Self::Asset3dImport {
                input: input.ok_or("asset3d-import requiert --input")?,
                kind: asset3d_type.ok_or("asset3d-import requiert --type")?,
                output: output.ok_or("asset3d-import requiert --output")?,
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

fn parse_u32_option(value: &str, option: &str) -> Result<u32> {
    value
        .parse()
        .map_err(|_| AppError::new(format!("{option} doit ├¬tre un entier non sign├®")))
}

fn parse_u64_option(value: &str, option: &str) -> Result<u64> {
    value
        .parse()
        .map_err(|_| AppError::new(format!("{option} doit ├¬tre un entier non sign├®")))
}

fn parse_percent_option(value: &str, option: &str) -> Result<u64> {
    let parsed = parse_u64_option(value, option)?;
    if parsed > 100_000 {
        return Err(format!("{option} doit ├¬tre compris entre 0 et 100000").into());
    }
    Ok(parsed)
}

pub fn help() -> &'static str {
    "Aetherion 0.1.0 — moteur de jeu CLI headless et déterministe\n\nUSAGE:\n  aetherion <COMMANDE> [OPTIONS]\n\nCOMMANDES:\n  init           Crée un projet déclaratif minimal\n  doctor         Vérifie la configuration et l'environnement\n  inspect        Émet le snapshot initial JSON\n  run            Exécute une simulation bornée\n  capture        Écrit une image PPM/PNG et son manifeste JSON\n  capture-multi  Publie atomiquement plusieurs vues\n  capture3d      Rend une scène 3D headless en PPM\n  play           Ouvre l'affichage Windows optionnel\n  replay-create  Crée un replay avec checkpoints configurables\n  replay-run     Rejoue et vérifie un replay v1 ou v2\n  diff           Compare deux snapshots ou manifestes JSON\n  visual-diff    Compare deux captures avec tolérances entières\n  scenario-run   Exécute un scénario agent-native borné\n  agent          Pilote un monde par JSONL sur stdin/stdout\n  schema         Liste ou affiche les schémas JSON publiés\n  scene          Liste ou affiche les scènes JSON\n  help           Affiche cette aide\n\nM4 prototype 3D:\n  capture3d --scene FILE --output FILE [--width N] [--height N] [--ticks N] [--animation ID] [--assets FILE] [--channels color,depth,normals,segmentation]\n\nM4-D:\n  visual-diff --baseline FILE --candidate FILE [--max-channel-delta N] [--max-different-pixels N] [--max-different-percent-milli N] [--report FILE]\n\nM4-H:\n  visual-diff3d --baseline-manifest FILE --candidate-manifest FILE --report FILE [--color-max-channel-delta N] [--color-max-different-pixels N] [--color-max-different-percent-milli N] [options depth/normals ├®quivalentes] [--segmentation-max-different-pixels N]\n\nM3:\n  capture --path DIR --ticks N --format ppm|png --output FILE [--assets FILE] [--scene ID] [--channels color,depth,normals,segmentation]\n  capture-multi --path DIR --views FILE --output-dir DIR [--ticks N] [--assets FILE] [--scene ID] [--channels color,depth,normals,segmentation]\n  scene list [--root PATH] | scene show ID [--root PATH]\n  play --path DIR [--max-ticks N] (requiert --features display)\n\nM2:\n  agent --path DIR --root DIR [--policy FILE] [--audit FILE]\n  schema list | schema show NOM\n\nCODES:\n  0 succès, 1 différence/assertion échouée, 2 usage/validation, 3 divergence/budget"
}

pub fn execute(command: Command) -> Result<Option<String>> {
    match command {
        Command::Help => Ok(Some(help().into())),
        Command::Init { path, force } => {
            fs::create_dir_all(&path)
                .map_err(|error| format!("création de {}: {error}", path.display()))?;
            let target = path.join(PROJECT_FILE);
            if target.exists() && !force {
                return Err(format!("{} existe déjà (utilisez --force)", target.display()).into());
            }
            fs::write(&target, Project::example())
                .map_err(|error| format!("écriture de {}: {error}", target.display()))?;
            Ok(Some(format!("Projet Aetherion créé: {}", target.display())))
        }
        Command::Doctor { path } => {
            let project = Project::load(&path)?;
            Ok(Some(format!(
                "OK: {} — {} entité(s), tick_rate={} Hz, seed={}",
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
                    "Simulation terminée: tick={}, entités={}, checksum={}",
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
                "Capture écrite: {} (manifeste: {})",
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
                "Captures multi-vues écrites: {}",
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
                "Capture 3D écrite: {} (manifeste: {})",
                output.display(),
                manifest.display()
            )))
        }
        Command::Asset3dImport {
            input,
            kind,
            output,
        } => {
            assets3d::import(&input, kind, &output)?;
            Ok(Some(format!("Ressource 3D importée: {}", output.display())))
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
                |error| format!("sérialisation du rapport: {error}"),
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
                |error| format!("sérialisation du rapport: {error}"),
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
                .map_err(|error| format!("sérialisation des scènes: {error}"))?,
        )),
        Command::SceneShow { root, id } => Ok(Some(
            serde_json::to_string_pretty(&scene::load(&root, &id)?)
                .map_err(|error| format!("sérialisation de la scène: {error}"))?,
        )),
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
}
