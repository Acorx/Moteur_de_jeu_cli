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

#[test]
fn certification_is_machine_readable_stable_and_atomic() {
    let directory = temporary_directory("m4-certification");
    std::fs::create_dir_all(&directory).unwrap();
    let first = directory.join("first.json");
    let second = directory.join("second.json");
    let binary = env!("CARGO_BIN_EXE_aetherion");

    for report in [&first, &second] {
        let output = Command::new(binary)
            .arg("certify-m4")
            .arg("--report")
            .arg(report)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(report).unwrap()).unwrap();
        assert_eq!(stdout, saved);
        assert_eq!(saved["schema"], "aetherion.m4-certification/v1");
        assert_eq!(saved["status"], "certified");
        assert_eq!(saved["checks"].as_array().unwrap().len(), 6);
        assert!(
            saved["checks"]
                .as_array()
                .unwrap()
                .iter()
                .all(|check| check["passed"] == true)
        );
    }

    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap()
    );
    let before = std::fs::read(&first).unwrap();
    let invalid = Command::new(binary).arg("certify-m4").output().unwrap();
    assert_eq!(invalid.status.code(), Some(2));
    assert_eq!(std::fs::read(&first).unwrap(), before);
    assert!(std::fs::read_dir(&directory).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".tmp")
    }));
    std::fs::remove_dir_all(directory).unwrap();
}
