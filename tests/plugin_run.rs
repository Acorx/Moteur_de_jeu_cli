#![cfg(feature = "plugin-runtime")]

use std::process::Command;

const RETURN_SEVEN: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x03,
    0x02, 0x01, 0x00, 0x07, 0x12, 0x01, 0x0e, 0x61, 0x65, 0x74, 0x68, 0x65, 0x72, 0x69, 0x6f, 0x6e,
    0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x07, 0x0b,
];

fn temporary_directory() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "aetherion-plugin-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn manifest() -> &'static str {
    r#"{
      "schema":"aetherion.plugin/v1",
      "id":"org.aetherion.cli-integration",
      "version":"1.0.0",
      "abi":{"major":1,"minimum_host_minor":1},
      "capabilities":[],
      "quotas":{"memory_bytes":65536,"fuel":1000,"io_read_bytes":0,"io_write_bytes":0,"files":0}
    }"#
}

#[test]
fn plugin_run_supports_dry_run_execution_and_atomic_report() {
    let root = temporary_directory();
    std::fs::create_dir_all(&root).unwrap();
    let manifest_path = root.join("plugin.json");
    let module_path = root.join("plugin.wasm");
    let report_path = root.join("report.json");
    std::fs::write(&manifest_path, manifest()).unwrap();
    std::fs::write(&module_path, RETURN_SEVEN).unwrap();
    let binary = env!("CARGO_BIN_EXE_aetherion");

    let dry_run = Command::new(binary)
        .args(["plugin", "run", "--manifest"])
        .arg(&manifest_path)
        .args(["--module"])
        .arg(&module_path)
        .args(["--dry-run", "--report"])
        .arg(&report_path)
        .output()
        .unwrap();
    assert!(
        dry_run.status.success(),
        "{}",
        String::from_utf8_lossy(&dry_run.stderr)
    );
    let planned: serde_json::Value = serde_json::from_slice(&dry_run.stdout).unwrap();
    assert_eq!(planned["schema"], "aetherion.plugin-run-report/v1");
    assert_eq!(planned["status"], "planned");
    assert_eq!(planned["return_code"], serde_json::Value::Null);

    let executed = Command::new(binary)
        .args(["plugin", "run", "--manifest"])
        .arg(&manifest_path)
        .args(["--module"])
        .arg(&module_path)
        .arg("--report")
        .arg(&report_path)
        .output()
        .unwrap();
    assert!(
        executed.status.success(),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).unwrap()).unwrap();
    assert_eq!(report["status"], "executed");
    assert_eq!(report["return_code"], 7);
    assert_eq!(report["io"]["write_bytes"], 0);
    assert!(
        !root
            .join(format!(".aetherion-plugin-run-{}.tmp", std::process::id()))
            .exists()
    );
    std::fs::remove_dir_all(root).unwrap();
}
