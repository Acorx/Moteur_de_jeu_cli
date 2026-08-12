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

fn write_manifest(path: &std::path::Path, minimum_host_minor: u32) {
    std::fs::write(
        path,
        format!(
            r#"{{"schema":"aetherion.plugin/v1","id":"org.example.abi","version":"1.0.0","abi":{{"major":1,"minimum_host_minor":{minimum_host_minor}}},"capabilities":[],"quotas":{{"memory_bytes":1024,"fuel":1000,"io_read_bytes":0,"io_write_bytes":0,"files":0}}}}"#
        ),
    )
    .unwrap();
}

#[test]
fn cli_enforces_minor_abi_policy_for_current_host() {
    let directory = temporary_directory("plugin-abi");
    std::fs::create_dir_all(&directory).unwrap();
    let binary = env!("CARGO_BIN_EXE_aetherion");

    let compatible_previous = directory.join("previous.plugin.json");
    write_manifest(&compatible_previous, 0);
    let previous = Command::new(binary)
        .args(["plugin", "validate"])
        .arg(&compatible_previous)
        .output()
        .unwrap();
    assert!(
        previous.status.success(),
        "{}",
        String::from_utf8_lossy(&previous.stderr)
    );
    let previous_report: serde_json::Value = serde_json::from_slice(&previous.stdout).unwrap();
    assert_eq!(previous_report["host_abi"]["major"], 1);
    assert_eq!(previous_report["host_abi"]["minor"], 1);
    assert_eq!(previous_report["compatibility"]["previous_host_minor"], 0);

    let compatible_current = directory.join("current.plugin.json");
    write_manifest(&compatible_current, 1);
    let current = Command::new(binary)
        .args(["plugin", "validate"])
        .arg(&compatible_current)
        .output()
        .unwrap();
    assert!(
        current.status.success(),
        "{}",
        String::from_utf8_lossy(&current.stderr)
    );

    let incompatible_future = directory.join("future.plugin.json");
    write_manifest(&incompatible_future, 2);
    let future = Command::new(binary)
        .args(["plugin", "validate"])
        .arg(&incompatible_future)
        .output()
        .unwrap();
    assert_eq!(future.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&future.stderr).contains("plugin_abi_incompatible"));

    std::fs::remove_dir_all(directory).unwrap();
}
