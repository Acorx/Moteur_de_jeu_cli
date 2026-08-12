#![cfg(feature = "plugin-runtime")]

use std::path::PathBuf;
use std::process::Command;

use aetherion::plugin_audit::{AuditOptions, audit};

const RETURN_SEVEN: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f, 0x03,
    0x02, 0x01, 0x00, 0x07, 0x12, 0x01, 0x0e, 0x61, 0x65, 0x74, 0x68, 0x65, 0x72, 0x69, 0x6f, 0x6e,
    0x5f, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x07, 0x0b,
];

fn temporary_directory() -> PathBuf {
    std::env::temp_dir().join(format!(
        "aetherion-plugin-audit-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn manifest() -> &'static str {
    r#"{"schema":"aetherion.plugin/v1","id":"org.aetherion.audit","version":"1.0.0","abi":{"major":1,"minimum_host_minor":1},"capabilities":[],"quotas":{"memory_bytes":65536,"fuel":1000,"io_read_bytes":0,"io_write_bytes":0,"files":0}}"#
}

#[test]
fn audit_report_is_versioned_machine_readable_and_golden() {
    let root = temporary_directory();
    std::fs::create_dir_all(&root).unwrap();
    let manifest_path = root.join("plugin.json");
    let module_path = root.join("plugin.wasm");
    let report_path = root.join("audit.json");
    std::fs::write(&manifest_path, manifest()).unwrap();
    std::fs::write(&module_path, RETURN_SEVEN).unwrap();

    let report = audit(AuditOptions {
        manifest: manifest_path.clone(),
        module: module_path.clone(),
        export: "aetherion_main".into(),
        report: Some(report_path.clone()),
    })
    .unwrap();
    let golden = include_str!("fixtures/plugin-audit-v1.json").replace("\r\n", "\n");
    assert_eq!(
        serde_json::to_string_pretty(&report).unwrap(),
        golden.trim_end()
    );
    assert_eq!(
        std::fs::read_to_string(report_path).unwrap(),
        format!("{}\n", golden.trim_end())
    );
    assert_eq!(report.status, "verified");
    assert!(
        !serde_json::to_string(&report)
            .unwrap()
            .contains(root.to_string_lossy().as_ref())
    );

    let cli_report_path = root.join("audit-cli.json");
    let output = Command::new(env!("CARGO_BIN_EXE_aetherion"))
        .args(["plugin", "audit", "--manifest"])
        .arg(&manifest_path)
        .args(["--module"])
        .arg(&module_path)
        .args(["--report"])
        .arg(&cli_report_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cli_value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(cli_value["schema"], "aetherion.plugin-audit/v1");
    assert_eq!(cli_value["status"], "verified");
    std::fs::remove_dir_all(root).unwrap();
}
