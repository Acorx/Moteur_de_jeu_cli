use std::process::Command;

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

fn project_with_collider() -> &'static str {
    r#"[project]
name = "physics-foundation"
format_version = 1

[simulation]
tick_rate = 60
seed = 7

[[entities]]
id = 1
name = "dynamic"
position = { x = 0, y = 0 }
velocity = { x = 2, y = 0 }
collider = { half_width = 1, half_height = 2, mass_milli = 1000, restitution_milli = 500 }

[[entities]]
id = 2
name = "background"
position = { x = 10, y = 0 }
velocity = { x = 0, y = 0 }
"#
}

fn colliding_project() -> &'static str {
    r#"[project]
name = "collision"
format_version = 1

[simulation]
tick_rate = 60
seed = 1

[[entities]]
id = 1
name = "left"
position = { x = 0, y = 0 }
velocity = { x = 1, y = 0 }
collider = { half_width = 1, half_height = 1, mass_milli = 1000, restitution_milli = 1000 }

[[entities]]
id = 2
name = "right"
position = { x = 1, y = 0 }
velocity = { x = 0, y = 0 }
collider = { half_width = 1, half_height = 1, mass_milli = 1000, restitution_milli = 1000 }
"#
}

#[test]
fn optional_collider_is_exposed_and_physics_system_is_observable() {
    let directory = temporary_directory("physics-foundation");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("aetherion.toml"), project_with_collider()).unwrap();
    let binary = env!("CARGO_BIN_EXE_aetherion");

    let inspect = Command::new(binary)
        .args(["inspect", "--path"])
        .arg(&directory)
        .output()
        .unwrap();
    assert!(
        inspect.status.success(),
        "{}",
        String::from_utf8_lossy(&inspect.stderr)
    );
    let initial: serde_json::Value = serde_json::from_slice(&inspect.stdout).unwrap();
    assert_eq!(initial["entities"][0]["collider"]["half_width"], 1);
    assert_eq!(initial["entities"][0]["collider"]["restitution_milli"], 500);
    assert!(initial["entities"][1].get("collider").is_none());

    let telemetry = directory.join("telemetry.json");
    let run = Command::new(binary)
        .args(["run", "--path"])
        .arg(&directory)
        .args(["--ticks", "2", "--json", "--telemetry"])
        .arg(&telemetry)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let final_state: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();
    assert_eq!(final_state["entities"][0]["position"]["x"], 4);
    assert_eq!(final_state["entities"][0]["collider"]["is_static"], false);

    let telemetry_value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&telemetry).unwrap()).unwrap();
    assert_eq!(
        telemetry_value["system_order"],
        serde_json::json!(["input", "movement", "physics"])
    );
    let physics = telemetry_value["systems"]
        .as_array()
        .unwrap()
        .iter()
        .find(|system| system["name"] == "physics")
        .unwrap();
    assert_eq!(physics["ticks"], 2);
    assert_eq!(physics["entities_visited"], 2);
    assert_eq!(physics["collisions_resolved"], 0);

    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn cli_resolves_canonical_dynamic_collision() {
    let directory = temporary_directory("physics-collision");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("aetherion.toml"), colliding_project()).unwrap();
    let telemetry = directory.join("telemetry.json");
    let output = Command::new(env!("CARGO_BIN_EXE_aetherion"))
        .args(["run", "--path"])
        .arg(&directory)
        .args(["--ticks", "1", "--json", "--telemetry"])
        .arg(&telemetry)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let snapshot: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(snapshot["entities"][0]["position"]["x"], 0);
    assert_eq!(snapshot["entities"][1]["position"]["x"], 2);
    assert_eq!(snapshot["entities"][0]["velocity"]["x"], 0);
    assert_eq!(snapshot["entities"][1]["velocity"]["x"], 1);
    let telemetry_value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(telemetry).unwrap()).unwrap();
    let physics = telemetry_value["systems"]
        .as_array()
        .unwrap()
        .iter()
        .find(|system| system["name"] == "physics")
        .unwrap();
    assert_eq!(physics["collisions_resolved"], 1);
    assert_eq!(physics["entities_modified"], 2);
    std::fs::remove_dir_all(directory).unwrap();
}
