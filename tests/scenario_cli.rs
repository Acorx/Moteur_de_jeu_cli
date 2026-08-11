use std::process::Command;

fn temporary_directory() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "aetherion-scenario-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn scenario(x: i64, ticks: u64, budget_ticks: u64) -> String {
    format!(
        r#"{{"schema":"aetherion.scenario/v1","project":{{"name":"hello-aetherion"}},"max_ticks":{ticks},"events":[],"assertions":[{{"id":"position","type":"entity_position","entity_id":1,"x":{x},"y":0}}],"budgets":{{"max_ticks":{budget_ticks},"max_events":0,"max_assertions":1,"max_input_bytes":65536,"max_output_bytes":262144}}}}"#
    )
}

#[test]
fn pass_fail_budget_report_audit_and_determinism() {
    let directory = temporary_directory();
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("aetherion.toml"),
        aetherion::project::Project::example(),
    )
    .unwrap();
    let pass = directory.join("pass.json");
    let fail = directory.join("fail.json");
    let budget = directory.join("budget.json");
    std::fs::write(&pass, scenario(3, 3, 3)).unwrap();
    std::fs::write(&fail, scenario(99, 3, 3)).unwrap();
    std::fs::write(&budget, scenario(3, 3, 2)).unwrap();
    let report = directory.join("report.json");
    let audit = directory.join("audit.jsonl");
    let binary = env!("CARGO_BIN_EXE_aetherion");
    let run = |input: &std::path::Path| {
        Command::new(binary)
            .args(["scenario-run", "--path"])
            .arg(&directory)
            .arg("--scenario")
            .arg(input)
            .arg("--report")
            .arg(&report)
            .arg("--audit")
            .arg(&audit)
            .output()
            .unwrap()
    };

    let first = run(&pass);
    assert!(first.status.success());
    let first_value: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(first_value["status"], "pass");
    assert_eq!(first_value["assertions"][0]["passed"], true);
    let saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
    assert_eq!(saved["schema"], "aetherion.scenario-report/v1");

    let second = run(&pass);
    let second_value: serde_json::Value = serde_json::from_slice(&second.stdout).unwrap();
    assert_eq!(first_value["run_id"], second_value["run_id"]);
    assert_eq!(first_value["final_state"], second_value["final_state"]);
    let lines = std::fs::read_to_string(&audit).unwrap();
    assert_eq!(lines.lines().count(), 2);
    for line in lines.lines() {
        let entry: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(entry["schema"], "aetherion.audit/v1");
        assert!(entry.get("timestamp").is_none());
    }

    assert_eq!(run(&fail).status.code(), Some(1));
    assert_eq!(run(&budget).status.code(), Some(3));
    let invalid = directory.join("invalid.json");
    std::fs::write(&invalid, "not json").unwrap();
    assert_eq!(run(&invalid).status.code(), Some(2));
    std::fs::remove_dir_all(directory).unwrap();
}
