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
fn bundle_is_deterministic_inspectable_and_excludes_its_output() {
    let directory = temporary_directory("bundle");
    let source = directory.join("source");
    std::fs::create_dir_all(source.join("nested")).unwrap();
    std::fs::write(source.join("z.txt"), b"zeta").unwrap();
    std::fs::write(source.join("nested/a.txt"), b"alpha").unwrap();
    let output = source.join("game.bundle.zip");
    let binary = env!("CARGO_BIN_EXE_aetherion");

    let created = Command::new(binary)
        .args(["bundle", "--path"])
        .arg(&source)
        .args(["--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(created.status.success(), "{:?}", created);
    let first = std::fs::read(&output).unwrap();
    let recreated = Command::new(binary)
        .args(["bundle", "--path"])
        .arg(&source)
        .args(["--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(recreated.status.success(), "{:?}", recreated);
    assert_eq!(first, std::fs::read(&output).unwrap());

    let inspected = Command::new(binary)
        .args(["bundle-inspect", "--input"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(inspected.status.success(), "{:?}", inspected);
    let report: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
    assert_eq!(report["schema"], "aetherion.bundle-inspect/v1");
    let paths: Vec<_> = report["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap())
        .collect();
    assert_eq!(paths[0], "aetherion.bundle.json");
    assert!(paths.contains(&"nested/a.txt"));
    assert!(paths.contains(&"z.txt"));
    assert!(!paths.contains(&"game.bundle.zip"));
    assert!(std::fs::read_dir(&source).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".tmp")
    }));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn bundle_inspect_rejects_malformed_archive() {
    let directory = temporary_directory("bundle-invalid");
    std::fs::create_dir_all(&directory).unwrap();
    let invalid = directory.join("invalid.zip");
    std::fs::write(&invalid, b"not a zip archive").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_aetherion"))
        .args(["bundle-inspect", "--input"])
        .arg(&invalid)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("bundle_zip_eocd_missing"));
    std::fs::remove_dir_all(directory).unwrap();
}
