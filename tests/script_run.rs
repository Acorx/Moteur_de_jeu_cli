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

fn run(
    binary: &str,
    script: &std::path::Path,
    report: &std::path::Path,
    dry_run: bool,
) -> std::process::Output {
    let mut command = Command::new(binary);
    command.args(["script-run", "--script"]).arg(script);
    if dry_run {
        command.arg("--dry-run");
    }
    command.args(["--report"]).arg(report).output().unwrap()
}

#[test]
fn substitutions_dry_run_and_reports_are_versioned_atomic_and_deterministic() {
    let directory = temporary_directory("script-run");
    std::fs::create_dir_all(&directory).unwrap();
    let script = directory.join("script.json");
    std::fs::write(
        &script,
        r#"{"schema":"aetherion.script/v1","vars":{"name":"world"},"commands":[["echo","hello {{name}}"],"false"],"budget":{"max_commands":2,"max_ticks_total":2},"on_error":"continue"}"#,
    )
    .unwrap();
    let first = directory.join("first.json");
    let second = directory.join("second.json");
    let binary = env!("CARGO_BIN_EXE_aetherion");

    for report in [&first, &second] {
        let output = run(binary, &script, report, true);
        assert!(output.status.success(), "{:?}", output);
        let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let saved: serde_json::Value =
            serde_json::from_slice(&std::fs::read(report).unwrap()).unwrap();
        assert_eq!(stdout, saved);
        assert_eq!(saved["schema"], "aetherion.script-report/v1");
        assert_eq!(saved["commands_consumed"], 2);
        assert_eq!(saved["ticks_consumed"], 2);
        assert_eq!(saved["results"][0]["args"][1], "hello world");
        assert_eq!(saved["ok"], true);
    }
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap()
    );
    assert!(std::fs::read_dir(&directory).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".tmp")
    }));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn stop_continue_and_budget_failures_keep_stable_codes_and_reports() {
    let directory = temporary_directory("script-controls");
    std::fs::create_dir_all(&directory).unwrap();
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let stop = directory.join("stop.json");
    let continue_script = directory.join("continue.json");
    let budget = directory.join("budget.json");
    std::fs::write(&stop, r#"{"schema":"aetherion.script/v1","commands":["false","true"],"budget":{"max_commands":2,"max_ticks_total":2},"on_error":"stop"}"#).unwrap();
    std::fs::write(&continue_script, r#"{"schema":"aetherion.script/v1","commands":["false","true"],"budget":{"max_commands":2,"max_ticks_total":2},"on_error":"continue"}"#).unwrap();
    std::fs::write(&budget, r#"{"schema":"aetherion.script/v1","commands":["true","true"],"budget":{"max_commands":2,"max_ticks_total":1},"on_error":"continue"}"#).unwrap();

    let stop_report = directory.join("stop-report.json");
    let stopped = run(binary, &stop, &stop_report, false);
    assert_eq!(stopped.status.code(), Some(1));
    let stopped_report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&stop_report).unwrap()).unwrap();
    assert_eq!(stopped_report["commands_consumed"], 1);
    assert_eq!(stopped_report["results"][0]["exit_code"], 1);

    let continue_report = directory.join("continue-report.json");
    let continued = run(binary, &continue_script, &continue_report, false);
    assert_eq!(continued.status.code(), Some(1));
    let continued_report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&continue_report).unwrap()).unwrap();
    assert_eq!(continued_report["commands_consumed"], 2);
    assert_eq!(continued_report["results"][1]["exit_code"], 0);

    let budget_report = directory.join("budget-report.json");
    let exhausted = run(binary, &budget, &budget_report, false);
    assert_eq!(exhausted.status.code(), Some(3));
    assert!(!budget_report.exists());
    std::fs::remove_dir_all(directory).unwrap();
}
