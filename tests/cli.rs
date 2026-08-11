use std::process::Command;

#[test]
fn help_lists_core_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_aetherion"))
        .arg("help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for command in [
        "init",
        "doctor",
        "inspect",
        "run",
        "capture",
        "replay-create",
        "replay-run",
        "diff",
        "scenario-run",
    ] {
        assert!(stdout.contains(command));
    }
}

#[test]
fn init_then_run_emits_json() {
    let directory = std::env::temp_dir().join(format!(
        "aetherion-cli-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let binary = env!("CARGO_BIN_EXE_aetherion");
    assert!(
        Command::new(binary)
            .args(["init", "--path"])
            .arg(&directory)
            .status()
            .unwrap()
            .success()
    );
    let output = Command::new(binary)
        .args(["run", "--path"])
        .arg(&directory)
        .args(["--ticks", "3", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["tick"], 3);
    assert_eq!(value["entities"][0]["position"]["x"], 3);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn capture_writes_deterministic_ppm_and_manifest() {
    let directory = std::env::temp_dir().join(format!(
        "aetherion-capture-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("aetherion.toml"),
        aetherion::project::Project::example(),
    )
    .unwrap();
    let first = directory.join("first.ppm");
    let second = directory.join("second.ppm");
    let binary = env!("CARGO_BIN_EXE_aetherion");
    for output in [&first, &second] {
        assert!(
            Command::new(binary)
                .args(["capture", "--path"])
                .arg(&directory)
                .args(["--ticks", "3", "--output"])
                .arg(output)
                .status()
                .unwrap()
                .success()
        );
    }
    let first_bytes = std::fs::read(&first).unwrap();
    assert!(first_bytes.starts_with(b"P6\n160 120\n255\n"));
    assert_eq!(first_bytes, std::fs::read(&second).unwrap());
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(format!("{}.json", first.display())).unwrap())
            .unwrap();
    assert_eq!(manifest["schema"], "aetherion.capture/v1");
    assert_eq!(manifest["tick"], 3);
    assert_eq!(manifest["dimensions"]["width"], 160);
    assert_eq!(manifest["visible_entities"][0]["name"], "player");
    assert!(manifest["world_checksum"].as_u64().is_some());
    assert!(manifest["image_checksum"].as_u64().is_some());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn png_multi_and_headless_play_are_bounded() {
    let directory = temporary_directory("m3-test");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("aetherion.toml"),
        aetherion::project::Project::example(),
    )
    .unwrap();
    let views = directory.join("views.json");
    std::fs::write(&views, r#"{"schema":"aetherion.capture-views/v1","views":[{"name":"main","width":32,"height":24,"format":"png","camera":{"x":0,"y":0,"pixels_per_unit":4}}]}"#).unwrap();
    let png = directory.join("image.png");
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let captured = Command::new(binary)
        .args(["capture", "--path"])
        .arg(&directory)
        .args(["--format", "png", "--output"])
        .arg(&png)
        .output()
        .unwrap();
    assert!(
        captured.status.success(),
        "{}",
        String::from_utf8_lossy(&captured.stderr)
    );
    assert!(
        std::fs::read(&png)
            .unwrap()
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );
    let output_dir = directory.join("multi");
    let multi = Command::new(binary)
        .args(["capture-multi", "--path"])
        .arg(&directory)
        .arg("--views")
        .arg(&views)
        .arg("--output-dir")
        .arg(&output_dir)
        .output()
        .unwrap();
    assert!(
        multi.status.success(),
        "{}",
        String::from_utf8_lossy(&multi.stderr)
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output_dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["schema"], "aetherion.capture-multi/v1");
    assert_eq!(manifest["views"][0]["name"], "main");
    if !cfg!(feature = "display") {
        let play = Command::new(binary)
            .args(["play", "--path"])
            .arg(&directory)
            .args(["--max-ticks", "1"])
            .output()
            .unwrap();
        assert_eq!(play.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&play.stderr).contains("--features display"));
    }
    std::fs::remove_dir_all(directory).unwrap();
}

fn temporary_directory(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "aetherion-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn replay_roundtrip_and_divergence_are_machine_readable() {
    let directory = temporary_directory("replay-test");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("aetherion.toml"),
        aetherion::project::Project::example(),
    )
    .unwrap();
    let events = directory.join("events.json");
    std::fs::write(&events, r#"{"schema":"aetherion.events/v1","events":[{"tick":1,"sequence":0,"entity_id":1,"command":"stop"}]}"#).unwrap();
    let replay = directory.join("test.replay.json");
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let created = Command::new(binary)
        .args(["replay-create", "--path"])
        .arg(&directory)
        .args(["--ticks", "4", "--events"])
        .arg(&events)
        .arg("--output")
        .arg(&replay)
        .output()
        .unwrap();
    assert!(created.status.success());
    let played = Command::new(binary)
        .args(["replay-run", "--path"])
        .arg(&directory)
        .arg("--replay")
        .arg(&replay)
        .output()
        .unwrap();
    assert!(played.status.success());
    let report: serde_json::Value = serde_json::from_slice(&played.stdout).unwrap();
    assert_eq!(report["status"], "identical");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&replay).unwrap()).unwrap();
    value["checksums"][2]["checksum"] = serde_json::json!(1);
    std::fs::write(&replay, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    let diverged = Command::new(binary)
        .args(["replay-run", "--path"])
        .arg(&directory)
        .arg("--replay")
        .arg(&replay)
        .output()
        .unwrap();
    assert_eq!(diverged.status.code(), Some(3));
    let report: serde_json::Value = serde_json::from_slice(&diverged.stdout).unwrap();
    assert_eq!(report["status"], "diverged");
    assert_eq!(report["divergence"]["tick"], 2);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn diff_codes_identical_different_and_invalid() {
    let directory = temporary_directory("diff-test");
    std::fs::create_dir_all(&directory).unwrap();
    let left = directory.join("left.json");
    let same = directory.join("same.json");
    let different = directory.join("different.json");
    let invalid = directory.join("invalid.json");
    let snapshot = r#"{"schema":"aetherion.snapshot/v1","tick":3,"entities":[{"id":1,"position":{"x":3,"y":0}}]}"#;
    std::fs::write(&left, snapshot).unwrap();
    std::fs::write(&same, snapshot).unwrap();
    std::fs::write(&different, r#"{"schema":"aetherion.snapshot/v1","tick":3,"entities":[{"id":1,"position":{"x":4,"y":0}}]}"#).unwrap();
    std::fs::write(&invalid, "not json").unwrap();
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let run = |right: &std::path::Path| {
        Command::new(binary)
            .arg("diff")
            .arg("--left")
            .arg(&left)
            .arg("--right")
            .arg(right)
            .output()
            .unwrap()
    };
    assert!(run(&same).status.success());
    let changed = run(&different);
    assert_eq!(changed.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&changed.stdout).unwrap();
    assert_eq!(report["differences"][0]["tick"], 3);
    assert_eq!(report["differences"][0]["entity_id"], 1);
    assert_eq!(run(&invalid).status.code(), Some(2));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn textured_capture_is_deterministic_and_differs_from_flat_fallback() {
    let fixtures = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let directory = temporary_directory("textured-capture");
    std::fs::create_dir_all(&directory).unwrap();
    let first = directory.join("first.ppm");
    let second = directory.join("second.ppm");
    let flat = directory.join("flat.ppm");
    let binary = env!("CARGO_BIN_EXE_aetherion");
    for output in [&first, &second] {
        let result = Command::new(binary)
            .args(["capture", "--path"])
            .arg(&fixtures)
            .args(["--scene", "textured", "--assets"])
            .arg(fixtures.join("assets.json"))
            .arg("--output")
            .arg(output)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let result = Command::new(binary)
        .args(["capture", "--path"])
        .arg(&fixtures)
        .args(["--scene", "textured", "--output"])
        .arg(&flat)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let first_bytes = std::fs::read(&first).unwrap();
    assert_eq!(first_bytes, std::fs::read(&second).unwrap());
    assert_ne!(first_bytes, std::fs::read(&flat).unwrap());
    let first_manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(format!("{}.json", first.display())).unwrap())
            .unwrap();
    let second_manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(format!("{}.json", second.display())).unwrap())
            .unwrap();
    assert_eq!(
        first_manifest["image_checksum"],
        second_manifest["image_checksum"]
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn scene_list_and_show_emit_json() {
    let fixtures = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let listed = Command::new(binary)
        .args(["scene", "list", "--root"])
        .arg(&fixtures)
        .output()
        .unwrap();
    assert!(listed.status.success());
    let list: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(list[0]["id"], "textured");
    let shown = Command::new(binary)
        .args(["scene", "show", "textured", "--root"])
        .arg(&fixtures)
        .output()
        .unwrap();
    assert!(shown.status.success());
    let scene: serde_json::Value = serde_json::from_slice(&shown.stdout).unwrap();
    assert_eq!(scene["id"], "textured");
}

#[test]
fn all_capture_channels_are_deterministic_and_default_has_no_aux() {
    let directory = temporary_directory("m4c-channels");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("aetherion.toml"),
        aetherion::project::Project::example(),
    )
    .unwrap();
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let first = directory.join("first.png");
    let second = directory.join("second.png");
    for output in [&first, &second] {
        let result = Command::new(binary)
            .args(["capture", "--path"])
            .arg(&directory)
            .args([
                "--format",
                "png",
                "--channels",
                "color,depth,normals,segmentation",
                "--output",
            ])
            .arg(output)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    for suffix in ["depth.pgm", "normals.png", "segmentation.png"] {
        assert_eq!(
            std::fs::read(directory.join(format!("first.{suffix}"))).unwrap(),
            std::fs::read(directory.join(format!("second.{suffix}"))).unwrap()
        );
    }
    let depth = std::fs::read(directory.join("first.depth.pgm")).unwrap();
    assert!(depth.starts_with(b"P5\n160 120\n65535\n"));
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(format!("{}.json", first.display())).unwrap())
            .unwrap();
    assert_eq!(manifest["channels"].as_array().unwrap().len(), 3);
    assert_eq!(manifest["segmentation_mapping"][0]["name"], "player");
    let default = directory.join("default.ppm");
    assert!(
        Command::new(binary)
            .args(["capture", "--path"])
            .arg(&directory)
            .arg("--output")
            .arg(&default)
            .status()
            .unwrap()
            .success()
    );
    assert!(!directory.join("default.depth.pgm").exists());
    let default_manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(format!("{}.json", default.display())).unwrap())
            .unwrap();
    assert!(default_manifest.get("channels").is_none());
    for invalid in ["", "color,color", "color,unknown", "depth"] {
        let output = directory.join(format!("invalid-{}.ppm", invalid.len()));
        assert!(
            !Command::new(binary)
                .args(["capture", "--path"])
                .arg(&directory)
                .arg("--channels")
                .arg(invalid)
                .arg("--output")
                .arg(output)
                .status()
                .unwrap()
                .success()
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn capture3d_is_cli_accessible_deterministic_and_strict() {
    let directory = temporary_directory("capture3d");
    std::fs::create_dir_all(&directory).unwrap();
    let scene = directory.join("scene.json");
    std::fs::write(
        &scene,
        r#"{"schema":"aetherion.scene3d/v1","camera":{"pixels_per_unit":2},"background":[1,2,3],"meshes":[{"id":"mesh","vertices":[{"x":-2,"y":-2,"z":1},{"x":2,"y":-2,"z":1},{"x":0,"y":2,"z":1}],"triangles":[[0,1,2]]}],"materials":[{"id":"red","color":[255,0,0],"opacity":1000}],"objects":[{"id":"object","mesh":"mesh","material":"red","transform":{"scale":[1000,1000,1000],"rotation":[0,0,0],"translation":[0,0,0]}}]}"#,
    )
    .unwrap();
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let first = directory.join("first.ppm");
    let second = directory.join("second.ppm");
    for output in [&first, &second] {
        let result = Command::new(binary)
            .arg("capture3d")
            .arg("--scene")
            .arg(&scene)
            .arg("--output")
            .arg(output)
            .args(["--width", "16", "--height", "12"])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap()
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(format!("{}.json", first.display())).unwrap())
            .unwrap();
    assert_eq!(manifest["schema"], "aetherion.capture3d/v1");
    assert_eq!(manifest["width"], 16);
    assert_eq!(manifest["height"], 12);
    assert!(manifest["visible_pixels"].as_u64().unwrap() > 0);

    let invalid = directory.join("invalid.json");
    std::fs::write(
        &invalid,
        r#"{"schema":"aetherion.scene3d/v1","triangles":[],"extra":true}"#,
    )
    .unwrap();
    let rejected = Command::new(binary)
        .arg("capture3d")
        .arg("--scene")
        .arg(&invalid)
        .arg("--output")
        .arg(directory.join("invalid.ppm"))
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("scene3d_invalid"));
    assert!(!directory.join("invalid.ppm").exists());
    assert!(!directory.join("invalid.ppm.json").exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn capture3d_animation_is_deterministic_loops_holds_and_rejects_invalid_input() {
    let directory = temporary_directory("capture3d-animation");
    std::fs::create_dir_all(&directory).unwrap();
    let scene = directory.join("animated-scene.json");
    std::fs::write(
        &scene,
        r#"{"schema":"aetherion.scene3d/v1","camera":{"pixels_per_unit":2},"background":[1,2,3],"meshes":[{"id":"mesh","vertices":[{"x":-2,"y":-2,"z":1},{"x":2,"y":-2,"z":1},{"x":0,"y":2,"z":1}],"triangles":[[0,1,2]]}],"materials":[{"id":"red","color":[255,0,0],"opacity":1000}],"objects":[{"id":"object","mesh":"mesh","material":"red"}],"animations":[{"id":"loop","duration_ticks":4,"looping":true,"tracks":[{"object":"object","keyframes":[{"tick":0,"transform":{"scale":[1000,1000,1000],"rotation":[0,0,0],"translation":[-2,0,0]}},{"tick":2,"transform":{"scale":[1000,1000,1000],"rotation":[0,0,0],"translation":[2,0,0]}}]}]},{"id":"hold","duration_ticks":4,"tracks":[{"object":"object","keyframes":[{"tick":0,"transform":{"scale":[1000,1000,1000],"rotation":[0,0,0],"translation":[-2,0,0]}},{"tick":4,"transform":{"scale":[1000,1000,1000],"rotation":[0,0,0],"translation":[2,0,0]}}]}]}]}"#,
    )
    .unwrap();
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let capture = |animation: &str, ticks: &str, output: &std::path::Path| {
        Command::new(binary)
            .arg("capture3d")
            .arg("--scene")
            .arg(&scene)
            .arg("--animation")
            .arg(animation)
            .arg("--ticks")
            .arg(ticks)
            .arg("--output")
            .arg(output)
            .args(["--width", "16", "--height", "12"])
            .output()
            .unwrap()
    };
    let first = directory.join("first.ppm");
    let second = directory.join("second.ppm");
    assert!(capture("loop", "2", &first).status.success());
    assert!(capture("loop", "2", &second).status.success());
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap()
    );
    let first_manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(format!("{}.json", first.display())).unwrap())
            .unwrap();
    let second_manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(format!("{}.json", second.display())).unwrap())
            .unwrap();
    assert_eq!(first_manifest, second_manifest);
    assert_eq!(first_manifest["schema"], "aetherion.capture3d/v1");
    assert_eq!(first_manifest["scene_schema"], "aetherion.scene3d/v1");
    assert_eq!(first_manifest["animation"], "loop");
    assert_eq!(first_manifest["tick"], 2);
    assert_eq!(first_manifest["width"], 16);
    assert_eq!(first_manifest["height"], 12);
    assert_eq!(first_manifest["triangles"], 1);
    assert!(first_manifest["visible_pixels"].as_u64().unwrap() > 0);
    let loop_zero = directory.join("loop-zero.ppm");
    let loop_wrapped = directory.join("loop-wrapped.ppm");
    assert!(capture("loop", "0", &loop_zero).status.success());
    assert!(capture("loop", "4", &loop_wrapped).status.success());
    assert_eq!(
        std::fs::read(&loop_zero).unwrap(),
        std::fs::read(&loop_wrapped).unwrap()
    );
    let hold_end = directory.join("hold-end.ppm");
    let hold_late = directory.join("hold-late.ppm");
    assert!(capture("hold", "4", &hold_end).status.success());
    assert!(capture("hold", "99", &hold_late).status.success());
    assert_eq!(
        std::fs::read(&hold_end).unwrap(),
        std::fs::read(&hold_late).unwrap()
    );
    let unknown = directory.join("unknown.ppm");
    let rejected = capture("missing", "1", &unknown);
    assert_eq!(rejected.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("animation_reference_missing"));
    assert!(!unknown.exists());
    assert!(!directory.join("unknown.ppm.json").exists());
    let invalid = directory.join("invalid-ticks.ppm");
    let rejected = capture("loop", "not-a-number", &invalid);
    assert_eq!(rejected.status.code(), Some(2));
    assert!(!invalid.exists());
    assert!(!directory.join("invalid-ticks.ppm.json").exists());
    std::fs::remove_dir_all(directory).unwrap();
}

fn write_assets3d_fixture(directory: &std::path::Path) -> std::path::PathBuf {
    let mesh = br#"{"schema":"aetherion.mesh3d/v1","mesh":{"id":"mesh","vertices":[{"x":-2,"y":-2,"z":1},{"x":2,"y":-2,"z":1},{"x":0,"y":2,"z":1}],"triangles":[[0,1,2]]}}"#;
    let material = br#"{"schema":"aetherion.material3d/v1","material":{"id":"red","color":[255,0,0],"opacity":1000}}"#;
    std::fs::write(directory.join("mesh.json"), mesh).unwrap();
    std::fs::write(directory.join("material.json"), material).unwrap();
    let manifest = serde_json::json!({
        "schema":"aetherion.assets3d/v1",
        "assets":[
            {"id":"mesh","path":"mesh.json","type":"mesh","size":mesh.len(),"checksum":aetherion::render::checksum_bytes(mesh)},
            {"id":"red","path":"material.json","type":"material","size":material.len(),"checksum":aetherion::render::checksum_bytes(material)}
        ]
    });
    let path = directory.join("assets.json");
    std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    path
}

fn assert_capture3d_rejected(
    binary: &str,
    scene: &std::path::Path,
    assets: &std::path::Path,
    output: &std::path::Path,
) {
    let result = Command::new(binary)
        .args(["capture3d", "--scene"])
        .arg(scene)
        .arg("--assets")
        .arg(assets)
        .arg("--output")
        .arg(output)
        .output()
        .unwrap();
    assert_eq!(
        result.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!output.exists());
    assert!(!std::path::PathBuf::from(format!("{}.json", output.display())).exists());
}

#[test]
fn capture3d_external_assets_are_deterministic_and_fail_atomically() {
    let directory = temporary_directory("m4f-assets");
    std::fs::create_dir_all(&directory).unwrap();
    let assets = write_assets3d_fixture(&directory);
    let scene = directory.join("scene.json");
    std::fs::write(&scene, br#"{"schema":"aetherion.scene3d/v1","camera":{"pixels_per_unit":2},"background":[1,2,3],"objects":[{"id":"object","mesh":"mesh","material":"red"}]}"#).unwrap();
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let first = directory.join("first.ppm");
    let second = directory.join("second.ppm");
    for output in [&first, &second] {
        let result = Command::new(binary)
            .args(["capture3d", "--scene"])
            .arg(&scene)
            .arg("--assets")
            .arg(&assets)
            .arg("--output")
            .arg(output)
            .args(["--width", "16", "--height", "12"])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap()
    );
    let first_manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(format!("{}.json", first.display())).unwrap())
            .unwrap();
    let second_manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(format!("{}.json", second.display())).unwrap())
            .unwrap();
    assert_eq!(first_manifest, second_manifest);
    assert_eq!(first_manifest["schema"], "aetherion.capture3d/v1");
    assert_eq!(first_manifest["triangles"], 1);

    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&assets).unwrap()).unwrap();
    for (label, mutate) in [
        ("checksum", (0usize, "checksum", serde_json::json!(0))),
        ("size", (0usize, "size", serde_json::json!(1))),
        (
            "missing",
            (0usize, "path", serde_json::json!("absent.json")),
        ),
        (
            "traversal",
            (0usize, "path", serde_json::json!("../mesh.json")),
        ),
    ] {
        let mut invalid = original.clone();
        invalid["assets"][mutate.0][mutate.1] = mutate.2;
        let invalid_path = directory.join(format!("{label}.json"));
        std::fs::write(&invalid_path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert_capture3d_rejected(
            binary,
            &scene,
            &invalid_path,
            &directory.join(format!("{label}.ppm")),
        );
    }

    let collision_scene = directory.join("collision.json");
    std::fs::write(&collision_scene, br#"{"schema":"aetherion.scene3d/v1","meshes":[{"id":"mesh"}],"objects":[{"id":"object","mesh":"mesh","material":"red"}]}"#).unwrap();
    assert_capture3d_rejected(
        binary,
        &collision_scene,
        &assets,
        &directory.join("collision.ppm"),
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn asset3d_import_is_canonical_atomic_and_rejects_invalid_input() {
    let directory = temporary_directory("m4f-import");
    std::fs::create_dir_all(&directory).unwrap();
    let input = directory.join("input.json");
    std::fs::write(&input, br#"{ "mesh": { "triangles": [], "vertices": [], "id": "mesh" }, "schema": "aetherion.mesh3d/v1" }"#).unwrap();
    let output = directory.join("canonical.json");
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let imported = Command::new(binary)
        .args(["asset3d-import", "--input"])
        .arg(&input)
        .args(["--type", "mesh", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        imported.status.success(),
        "{}",
        String::from_utf8_lossy(&imported.stderr)
    );
    let bytes = std::fs::read(&output).unwrap();
    assert!(bytes.ends_with(b"\n"));
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["schema"], "aetherion.mesh3d/v1");
    assert_eq!(value["mesh"]["id"], "mesh");
    let before = bytes.clone();
    let collision = Command::new(binary)
        .args(["asset3d-import", "--input"])
        .arg(&input)
        .args(["--type", "mesh", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert_eq!(collision.status.code(), Some(2));
    assert_eq!(std::fs::read(&output).unwrap(), before);
    let invalid = directory.join("invalid.json");
    std::fs::write(&invalid, b"not json").unwrap();
    let absent = directory.join("absent.json");
    let rejected = Command::new(binary)
        .args(["asset3d-import", "--input"])
        .arg(&invalid)
        .args(["--type", "mesh", "--output"])
        .arg(&absent)
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(2));
    assert!(!absent.exists());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn visual_diff3d_covers_channels_codes_atomic_report_and_determinism() {
    let directory = temporary_directory("visual-diff3d");
    std::fs::create_dir_all(&directory).unwrap();
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let scene = directory.join("scene.json");
    std::fs::write(&scene, br#"{"schema":"aetherion.scene3d/v1","camera":{"pixels_per_unit":2},"background":[1,2,3],"meshes":[{"id":"mesh","vertices":[{"x":-2,"y":-2,"z":1},{"x":2,"y":-2,"z":1},{"x":0,"y":2,"z":1}],"triangles":[[0,1,2]]}],"materials":[{"id":"red","color":[255,0,0],"opacity":1000}],"objects":[{"id":"object","mesh":"mesh","material":"red"}]}"#).unwrap();
    let baseline = directory.join("baseline.ppm");
    let candidate = directory.join("candidate.ppm");
    for output in [&baseline, &candidate] {
        let result = Command::new(binary)
            .args(["capture3d", "--scene"])
            .arg(&scene)
            .arg("--output")
            .arg(output)
            .args([
                "--width",
                "8",
                "--height",
                "6",
                "--channels",
                "color,depth,normals,segmentation",
            ])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let baseline_manifest = std::path::PathBuf::from(format!("{}.json", baseline.display()));
    let candidate_manifest = std::path::PathBuf::from(format!("{}.json", candidate.display()));
    let report = directory.join("report.json");
    let run = |candidate_manifest: &std::path::Path, report: &std::path::Path, extra: &[&str]| {
        let mut command = Command::new(binary);
        command
            .arg("visual-diff3d")
            .arg("--baseline-manifest")
            .arg(&baseline_manifest)
            .arg("--candidate-manifest")
            .arg(candidate_manifest)
            .arg("--report")
            .arg(report)
            .args(extra)
            .output()
            .unwrap()
    };
    let first = run(&candidate_manifest, &report, &[]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, std::fs::read(&report).unwrap());
    let second = run(&candidate_manifest, &report, &[]);
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(
        std::fs::read_dir(&directory)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry
                .file_name()
                .to_string_lossy()
                .contains("aetherion-tmp"))
            .count(),
        0
    );

    let mutate_ppm = |path: &std::path::Path, delta: u8| {
        let mut bytes = std::fs::read(path).unwrap();
        let index = bytes.iter().position(|byte| *byte == b'\n').unwrap();
        let index = bytes[index + 1..]
            .iter()
            .position(|byte| *byte == b'\n')
            .unwrap()
            + index
            + 1;
        let index = bytes[index + 1..]
            .iter()
            .position(|byte| *byte == b'\n')
            .unwrap()
            + index
            + 2;
        bytes[index] = bytes[index].saturating_add(delta);
        std::fs::write(path, bytes).unwrap();
    };
    mutate_ppm(&candidate, 1);
    mutate_ppm(&directory.join("candidate.normals.ppm"), 1);
    let mut depth = std::fs::read(directory.join("candidate.depth.pgm")).unwrap();
    let pixel = depth.len() - 1;
    depth[pixel] = depth[pixel].saturating_add(1);
    std::fs::write(directory.join("candidate.depth.pgm"), depth).unwrap();
    let strict = run(&candidate_manifest, &report, &[]);
    assert_eq!(strict.status.code(), Some(1));
    let tolerant = run(
        &candidate_manifest,
        &report,
        &[
            "--color-max-channel-delta",
            "1",
            "--depth-max-channel-delta",
            "1",
            "--normals-max-channel-delta",
            "1",
        ],
    );
    assert!(
        tolerant.status.success(),
        "{}",
        String::from_utf8_lossy(&tolerant.stderr)
    );

    mutate_ppm(&directory.join("candidate.segmentation.ppm"), 1);
    let segmented = run(
        &candidate_manifest,
        &report,
        &[
            "--color-max-channel-delta",
            "1",
            "--depth-max-channel-delta",
            "1",
            "--normals-max-channel-delta",
            "1",
        ],
    );
    assert_eq!(segmented.status.code(), Some(1));
    let segmented_json: serde_json::Value = serde_json::from_slice(&segmented.stdout).unwrap();
    assert_eq!(segmented_json["segmentation_differences"][0]["pixels"], 1);
    assert!(
        segmented_json["segmentation_differences"][0]
            .get("baseline_id")
            .is_some()
    );
    assert!(
        segmented_json["segmentation_differences"][0]
            .get("candidate_id")
            .is_some()
    );
    assert!(
        run(
            &candidate_manifest,
            &report,
            &[
                "--color-max-channel-delta",
                "1",
                "--depth-max-channel-delta",
                "1",
                "--normals-max-channel-delta",
                "1",
                "--segmentation-max-different-pixels",
                "1"
            ]
        )
        .status
        .success()
    );

    let missing_manifest = directory.join("missing.ppm.json");
    std::fs::copy(&candidate, directory.join("missing.ppm")).unwrap();
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&candidate_manifest).unwrap()).unwrap();
    manifest["channels"].as_array_mut().unwrap().pop();
    std::fs::write(&missing_manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert_eq!(run(&missing_manifest, &report, &[]).status.code(), Some(2));
    manifest["channels"][0]["encoding"] = serde_json::json!("wrong");
    std::fs::write(&missing_manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert_eq!(run(&missing_manifest, &report, &[]).status.code(), Some(2));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn visual_diff_exit_codes_tolerances_and_report_are_deterministic() {
    let directory = temporary_directory("visual-diff");
    std::fs::create_dir_all(&directory).unwrap();
    let baseline = directory.join("baseline.ppm");
    let same = directory.join("same.ppm");
    let changed = directory.join("changed.ppm");
    let malformed = directory.join("malformed.ppm");
    let report_path = directory.join("report.json");
    std::fs::write(&baseline, b"P6\n2 1\n255\n\x01\x02\x03\x04\x05\x06").unwrap();
    std::fs::copy(&baseline, &same).unwrap();
    std::fs::write(&changed, b"P6\n2 1\n255\n\x02\x02\x03\x04\x05\x06").unwrap();
    std::fs::write(&malformed, b"P6\n2 1\n255\n\x01").unwrap();
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let run = |candidate: &std::path::Path, extra: &[&str]| {
        let mut command = Command::new(binary);
        command
            .arg("visual-diff")
            .arg("--baseline")
            .arg(&baseline)
            .arg("--candidate")
            .arg(candidate);
        command.args(extra).output().unwrap()
    };
    assert!(run(&same, &[]).status.success());
    let strict = run(&changed, &[]);
    assert_eq!(strict.status.code(), Some(1));
    let strict_json: serde_json::Value = serde_json::from_slice(&strict.stdout).unwrap();
    assert_eq!(strict_json["schema"], "aetherion.visual-diff/v1");
    assert_eq!(strict_json["different_pixels"], 1);
    assert!(
        run(&changed, &["--max-channel-delta", "1"])
            .status
            .success()
    );
    assert_eq!(run(&malformed, &[]).status.code(), Some(2));
    let with_report = Command::new(binary)
        .arg("visual-diff")
        .arg("--baseline")
        .arg(&baseline)
        .arg("--candidate")
        .arg(&same)
        .arg("--report")
        .arg(&report_path)
        .output()
        .unwrap();
    assert!(with_report.status.success());
    let stdout: serde_json::Value = serde_json::from_slice(&with_report.stdout).unwrap();
    let file: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).unwrap()).unwrap();
    assert_eq!(stdout, file);
    std::fs::remove_dir_all(directory).unwrap();
}
