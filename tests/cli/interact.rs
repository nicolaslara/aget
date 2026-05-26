use std::fs;

use assert_cmd::Command;

#[test]
fn interact_dry_run_writes_audit_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let actions = temp.path().join("actions.json");
    fs::write(
        &actions,
        r#"{
          "schema_version": "aget.actions.v1",
          "actions": [
            {"type": "wait", "duration_ms": 1},
            {"type": "type", "selector": "input[name=token]", "text": "secret-token", "sensitive": true}
          ]
        }"#,
    )
    .unwrap();

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", temp.path().join("home"))
        .args([
            "--envelope",
            "json",
            "interact",
            "https://example.com/app",
            "--actions",
        ])
        .arg(&actions)
        .args(["--allow-actions", "--allow-sensitive-input", "--dry-run"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "interact");
    assert_eq!(envelope["data"]["actions"]["total"], 2);
    assert_eq!(envelope["data"]["actions"]["succeeded"], 2);

    let request_path = envelope["data"]["artifacts"]["actions_request"]
        .as_str()
        .unwrap();
    let result_path = envelope["data"]["artifacts"]["actions_result"]
        .as_str()
        .unwrap();
    let metadata_path = envelope["data"]["artifacts"]["metadata"].as_str().unwrap();
    assert!(std::path::Path::new(request_path).exists());
    assert!(std::path::Path::new(result_path).exists());
    assert!(std::path::Path::new(metadata_path).exists());

    let request = fs::read_to_string(request_path).unwrap();
    assert!(!request.contains("secret-token"));
    assert!(request.contains("<redacted>"));

    let result: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(result_path).unwrap()).unwrap();
    assert_eq!(result["actions"][0]["status"], "dry_run");
    assert_eq!(result["actions"][1]["status"], "dry_run");

    let run_id = envelope["data"]["run_id"].as_str().unwrap();
    let inspect_output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", temp.path().join("home"))
        .args(["--envelope", "json", "artifacts", "inspect", run_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect: serde_json::Value = serde_json::from_slice(&inspect_output).unwrap();
    let kinds = inspect["data"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|file| file["kind"].as_str())
        .collect::<Vec<_>>();
    assert!(kinds.contains(&"actions-request"));
    assert!(kinds.contains(&"actions-result"));
}

#[test]
fn interact_unimplemented_executor_writes_failure_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let actions = temp.path().join("actions.json");
    fs::write(
        &actions,
        r#"{"schema_version":"aget.actions.v1","actions":[{"type":"wait","duration_ms":1}]}"#,
    )
    .unwrap();

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", temp.path().join("home"))
        .args([
            "--envelope",
            "json",
            "interact",
            "https://example.com/app",
            "--actions",
        ])
        .arg(&actions)
        .arg("--allow-actions")
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], "backend_unavailable");
    assert_eq!(envelope["data"]["actions"]["failed_index"], 0);

    let result_path = envelope["data"]["artifacts"]["actions_result"]
        .as_str()
        .unwrap();
    let result: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(result_path).unwrap()).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["actions"][0]["status"], "failed");
}

#[test]
fn interact_rejects_missing_consent_before_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let actions = temp.path().join("actions.json");
    fs::write(
        &actions,
        r#"{"schema_version":"aget.actions.v1","actions":[{"type":"wait","duration_ms":1}]}"#,
    )
    .unwrap();

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", temp.path().join("home"))
        .args([
            "--envelope",
            "json",
            "interact",
            "https://example.com/app",
            "--actions",
        ])
        .arg(&actions)
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["command"], "interact");
    assert_eq!(envelope["error"]["code"], "usage_error");
    assert!(envelope["error"]["message"]
        .as_str()
        .unwrap()
        .contains("--allow-actions"));
}
