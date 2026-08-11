use std::io::Write;
use std::process::{Command, Stdio};

fn run(lines: &str, policy: Option<&std::path::Path>) -> Vec<serde_json::Value> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut command = Command::new(env!("CARGO_BIN_EXE_aetherion"));
    command
        .args(["agent", "--path"])
        .arg(root.join("demo"))
        .arg("--root")
        .arg(root);
    if let Some(policy) = policy {
        command.arg("--policy").arg(policy);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(lines.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn request(id: &str, method: &str, params: serde_json::Value) -> String {
    serde_json::json!({"schema":"aetherion.agent-request/v1","request_id":id,"method":method,"params":params}).to_string() + "\n"
}

fn isolated_root(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "aetherion-agent-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn run_with_root(
    lines: &str,
    root: &std::path::Path,
    policy: Option<&std::path::Path>,
) -> Vec<serde_json::Value> {
    let project = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut command = Command::new(env!("CARGO_BIN_EXE_aetherion"));
    command
        .args(["agent", "--path"])
        .arg(project.join("demo"))
        .arg("--root")
        .arg(root);
    if let Some(policy) = policy {
        command.arg("--policy").arg(policy);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(lines.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn views(count: usize) -> serde_json::Value {
    serde_json::json!({
        "schema": "aetherion.capture-views/v1",
        "views": (0..count).map(|index| serde_json::json!({
            "name": format!("view-{index}"),
            "width": 8,
            "height": 8,
            "camera": {"x": 0, "y": 0, "pixels_per_unit": 1},
            "format": "ppm"
        })).collect::<Vec<_>>()
    })
}

fn assert_no_staging(root: &std::path::Path) {
    assert!(std::fs::read_dir(root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("aetherion-staging")
    }));
}

#[test]
fn invalid_line_recovers_and_handshake_is_clean_jsonl() {
    let values = run(
        &(String::from("not-json\n") + &request("h", "handshake", serde_json::json!({}))),
        None,
    );
    assert_eq!(values.len(), 2);
    assert_eq!(values[0]["error"]["code"], "invalid_request");
    assert_eq!(values[1]["request_id"], "h");
    assert_eq!(values[1]["result"]["network"], false);
}

#[test]
fn dry_run_commit_stale_and_rollback_are_atomic() {
    let mut input = request("create", "session.create", serde_json::json!({}));
    input += &request(
        "dry",
        "transaction.execute",
        serde_json::json!({"session_id":"session-1","dry_run":true,"expected_revision":0,"operations":[{"method":"world.step","params":{"ticks":2,"events":[]}}]}),
    );
    input += &request(
        "inspect0",
        "world.inspect",
        serde_json::json!({"session_id":"session-1"}),
    );
    input += &request(
        "commit",
        "transaction.execute",
        serde_json::json!({"session_id":"session-1","expected_revision":0,"operations":[{"method":"world.step","params":{"ticks":2,"events":[]}}]}),
    );
    input += &request(
        "stale",
        "world.step",
        serde_json::json!({"session_id":"session-1","ticks":1,"events":[],"expected_revision":0}),
    );
    input += &request(
        "bad",
        "transaction.execute",
        serde_json::json!({"session_id":"session-1","expected_revision":1,"operations":[{"method":"input.apply","params":{"events":[{"tick":0,"sequence":0,"entity_id":999,"command":"stop"}]}}]}),
    );
    input += &request(
        "inspect1",
        "world.inspect",
        serde_json::json!({"session_id":"session-1"}),
    );
    let values = run(&input, None);
    assert_eq!(values[1]["result"]["committed"], false);
    assert_eq!(values[2]["result"]["snapshot"]["tick"], 0);
    assert_eq!(values[3]["result"]["committed"], true);
    assert_eq!(values[4]["error"]["code"], "stale_revision");
    assert_eq!(values[5]["error"]["code"], "transaction_aborted");
    assert_eq!(values[6]["result"]["snapshot"]["tick"], 2);
}

#[test]
fn transaction_capture_multi_commit_publishes_images_and_manifest() {
    let root = isolated_root("commit");
    let input = request("create", "session.create", serde_json::json!({}))
        + &request(
            "commit",
            "transaction.execute",
            serde_json::json!({
                "session_id":"session-1","expected_revision":0,"operations":[
                    {"method":"world.step","params":{"ticks":1}},
                    {"method":"capture.multi","params":{"output_dir":"batch","views":views(2)}}
                ]
            }),
        );
    let values = run_with_root(&input, &root, None);
    assert_eq!(values[1]["result"]["committed"], true);
    assert!(root.join("batch/view-0.ppm").is_file());
    assert!(root.join("batch/view-1.ppm").is_file());
    assert!(root.join("batch/manifest.json").is_file());
    assert_no_staging(&root);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn transaction_capture_multi_dry_run_has_no_files_or_mutation() {
    let root = isolated_root("dry");
    let input = request("create", "session.create", serde_json::json!({}))
        + &request(
            "dry",
            "transaction.execute",
            serde_json::json!({
                "session_id":"session-1","dry_run":true,"expected_revision":0,"operations":[
                    {"method":"world.step","params":{"ticks":2}},
                    {"method":"capture.multi","params":{"output_dir":"batch","views":views(2)}}
                ]
            }),
        )
        + &request(
            "inspect",
            "world.inspect",
            serde_json::json!({"session_id":"session-1"}),
        );
    let values = run_with_root(&input, &root, None);
    assert_eq!(values[1]["result"]["committed"], false);
    assert_eq!(values[2]["result"]["revision"], 0);
    assert_eq!(values[2]["result"]["snapshot"]["tick"], 0);
    assert!(!root.join("batch").exists());
    assert_no_staging(&root);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn transaction_capture_multi_rolls_back_when_later_operation_fails() {
    let root = isolated_root("rollback");
    let input = request("create", "session.create", serde_json::json!({}))
        + &request(
            "bad",
            "transaction.execute",
            serde_json::json!({
                "session_id":"session-1","expected_revision":0,"operations":[
                    {"method":"capture.multi","params":{"output_dir":"batch","views":views(2)}},
                    {"method":"input.apply","params":{"events":[{"tick":0,"sequence":0,"entity_id":999,"command":"stop"}]}}
                ]
            }),
        )
        + &request(
            "inspect",
            "world.inspect",
            serde_json::json!({"session_id":"session-1"}),
        );
    let values = run_with_root(&input, &root, None);
    assert_eq!(values[1]["error"]["code"], "transaction_aborted");
    assert_eq!(values[2]["result"]["revision"], 0);
    assert!(!root.join("batch").exists());
    assert_no_staging(&root);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn transaction_capture_multi_counts_each_view_against_quota() {
    let root = isolated_root("quota");
    let policy = root.join("policy.json");
    std::fs::write(&policy, serde_json::json!({
        "schema":"aetherion.capability-policy/v1",
        "capabilities":{"project_read":true,"world_mutate":true,"capture":true,"file_write":true},
        "limits":{"max_line_bytes":1048576,"max_operations":64,"max_ticks_per_request":10000,"max_events":10000,"max_captures":1,"max_output_bytes":4194304,"max_audit_bytes":4194304}
    }).to_string()).unwrap();
    let input = request("create", "session.create", serde_json::json!({}))
        + &request(
            "quota",
            "transaction.execute",
            serde_json::json!({
                "session_id":"session-1","operations":[{"method":"capture.multi","params":{"output_dir":"batch","views":views(2)}}]
            }),
        );
    let values = run_with_root(&input, &root, Some(&policy));
    assert_eq!(values[1]["error"]["code"], "transaction_aborted");
    assert_eq!(values[1]["error"]["details"]["cause"], "quota_exceeded");
    assert!(!root.join("batch").exists());
    assert_no_staging(&root);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn transaction_capture_multi_rejects_path_traversal_without_partials() {
    let root = isolated_root("traversal");
    let outside = root.parent().unwrap().join("escaped-aetherion-batch");
    let _ = std::fs::remove_dir_all(&outside);
    let input = request("create", "session.create", serde_json::json!({}))
        + &request(
            "escape",
            "transaction.execute",
            serde_json::json!({
                "session_id":"session-1","operations":[{"method":"capture.multi","params":{"output_dir":"../escaped-aetherion-batch","views":views(1)}}]
            }),
        );
    let values = run_with_root(&input, &root, None);
    assert_eq!(values[1]["error"]["code"], "transaction_aborted");
    assert!(!outside.exists());
    assert_no_staging(&root);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn policy_denies_mutation_and_schema_cli_works() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let policy = root.join("demo/agent-policy-readonly.json");
    let input = request("create", "session.create", serde_json::json!({}))
        + &request(
            "step",
            "world.step",
            serde_json::json!({"session_id":"session-1","ticks":1,"events":[]}),
        );
    let values = run(&input, Some(&policy));
    assert_eq!(values[1]["error"]["code"], "capability_denied");
    let output = Command::new(env!("CARGO_BIN_EXE_aetherion"))
        .args(["schema", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value["schemas"].as_array().unwrap().len() >= 10);
}
