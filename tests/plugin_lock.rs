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

fn write_manifest(path: &std::path::Path, id: &str) {
    std::fs::write(
        path,
        format!(
            r#"{{"schema":"aetherion.plugin/v1","id":"{id}","version":"1.0.0","abi":{{"major":1,"minimum_host_minor":0}},"capabilities":["telemetry_write","asset_read"],"quotas":{{"memory_bytes":1024,"fuel":1000,"io_read_bytes":0,"io_write_bytes":0,"files":0}}}}"#
        ),
    )
    .unwrap();
}

#[test]
fn resolve_writes_canonical_lock_and_lock_check_detects_checksum_change() {
    let directory = temporary_directory("plugin-lock");
    let plugins = directory.join("plugins");
    std::fs::create_dir_all(&plugins).unwrap();
    write_manifest(&plugins.join("z.plugin.json"), "org.example.zeta");
    write_manifest(&plugins.join("a.plugin.json"), "org.example.alpha");
    let lockfile = directory.join("plugins.lock.json");
    let binary = env!("CARGO_BIN_EXE_aetherion");

    let resolve = Command::new(binary)
        .args(["plugin", "resolve", "--dir"])
        .arg(&plugins)
        .args(["--lockfile"])
        .arg(&lockfile)
        .output()
        .unwrap();
    assert!(resolve.status.success(), "{:?}", resolve);
    let saved = std::fs::read(&lockfile).unwrap();
    assert!(saved.ends_with(b"\n"));
    let lock: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    assert_eq!(lock["schema"], "aetherion.plugin-lock/v1");
    assert_eq!(lock["plugins"][0]["id"], "org.example.alpha");
    assert_eq!(lock["plugins"][1]["id"], "org.example.zeta");
    assert_eq!(lock["plugins"][0]["capabilities"][0], "asset_read");

    let check = Command::new(binary)
        .args(["plugin", "lock-check", "--dir"])
        .arg(&plugins)
        .args(["--lockfile"])
        .arg(&lockfile)
        .output()
        .unwrap();
    assert!(check.status.success(), "{:?}", check);
    assert_eq!(std::fs::read(&lockfile).unwrap(), saved);

    write_manifest(&plugins.join("a.plugin.json"), "org.example.alpha");
    std::fs::write(
        plugins.join("a.plugin.json"),
        format!(
            "{}\n",
            String::from_utf8(std::fs::read(plugins.join("a.plugin.json")).unwrap()).unwrap()
        ),
    )
    .unwrap();
    let diverged = Command::new(binary)
        .args(["plugin", "lock-check", "--dir"])
        .arg(&plugins)
        .args(["--lockfile"])
        .arg(&lockfile)
        .output()
        .unwrap();
    assert_eq!(diverged.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&diverged.stdout).unwrap();
    assert_eq!(report["schema"], "aetherion.plugin-lock-check/v1");
    assert_eq!(report["status"], "diverged");
    assert_ne!(
        report["expected"][0]["checksum_fnv1a"],
        report["actual"][0]["checksum_fnv1a"]
    );
    std::fs::remove_dir_all(directory).unwrap();
}
