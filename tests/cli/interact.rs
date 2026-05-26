use std::fs;

use assert_cmd::Command;

use super::support::{
    mock_current_tab_interact_cdp_server, mock_current_tab_interact_cdp_server_with_page,
};

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
fn interact_session_backed_executor_writes_deferred_failure_artifacts() {
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
        .args([
            "--allow-actions",
            "--session",
            "app",
            "--allow-private-content",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], "action_not_supported");
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
fn interact_current_tab_executes_mock_cdp_actions() {
    let temp = tempfile::tempdir().unwrap();
    let actions = temp.path().join("actions.json");
    fs::write(
        &actions,
        r#"{
          "schema_version": "aget.actions.v1",
          "actions": [
            {"type": "wait", "duration_ms": 1},
            {"type": "click", "selector": "button#open"},
            {"type": "type", "selector": "input[name=q]", "text": "docs"},
            {"type": "select", "selector": "select[name=version]", "value": "stable"},
            {"type": "submit", "selector": "form#search", "confirm": true}
          ]
        }"#,
    )
    .unwrap();
    let (port, server) = mock_current_tab_interact_cdp_server(
        "https://example.com/app",
        vec![
            serde_json::json!({"ok": true}),
            serde_json::json!({"ok": true}),
            serde_json::json!({"ok": true}),
            serde_json::json!({"ok": true}),
        ],
    );

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", temp.path().join("home"))
        .args([
            "--envelope",
            "json",
            "interact",
            "current-tab",
            "--cdp-port",
            &port.to_string(),
            "--actions",
        ])
        .arg(&actions)
        .args([
            "--allow-actions",
            "--allow-private-content",
            "--allow-submit",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "interact");
    assert_eq!(envelope["data"]["actions"]["total"], 5);
    assert_eq!(envelope["data"]["actions"]["succeeded"], 5);
    assert_eq!(envelope["data"]["final_url"], "https://example.com/app");

    let result_path = envelope["data"]["artifacts"]["actions_result"]
        .as_str()
        .unwrap();
    let result: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(result_path).unwrap()).unwrap();
    let statuses = result["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|action| action["status"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(statuses, vec!["ok", "ok", "ok", "ok", "ok"]);
    server.join().unwrap();
}

#[test]
fn interact_current_tab_reports_action_failure_from_cdp() {
    let temp = tempfile::tempdir().unwrap();
    let actions = temp.path().join("actions.json");
    fs::write(
        &actions,
        r#"{
          "schema_version": "aget.actions.v1",
          "actions": [
            {"type": "click", "selector": "button#missing"}
          ]
        }"#,
    )
    .unwrap();
    let (port, server) = mock_current_tab_interact_cdp_server(
        "https://example.com/app",
        vec![serde_json::json!({
            "ok": false,
            "code": "selector_not_found",
            "message": "click selector matched no elements"
        })],
    );

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", temp.path().join("home"))
        .args([
            "--envelope",
            "json",
            "interact",
            "current-tab",
            "--cdp-port",
            &port.to_string(),
            "--actions",
        ])
        .arg(&actions)
        .args(["--allow-actions", "--allow-private-content"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], "selector_not_found");
    assert_eq!(envelope["data"]["actions"]["failed_index"], 0);

    let result_path = envelope["data"]["artifacts"]["actions_result"]
        .as_str()
        .unwrap();
    let result: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(result_path).unwrap()).unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(result["actions"][0]["status"], "failed");
    assert_eq!(result["actions"][0]["error"]["code"], "selector_not_found");
    server.join().unwrap();
}

#[test]
fn interact_current_tab_capture_and_extract_write_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let actions = temp.path().join("actions.json");
    fs::write(
        &actions,
        r#"{
          "schema_version": "aget.actions.v1",
          "actions": [
            {"type": "capture", "name": "after", "html": true, "screenshot": true, "sensitive": true},
            {"type": "extract", "name": "main", "selector": "main", "content_format": "markdown", "sensitive": true}
          ]
        }"#,
    )
    .unwrap();
    let (port, server) = mock_current_tab_interact_cdp_server_with_page(
        "https://example.com/app?secret=query-token#secret-fragment",
        "<html><body><nav>skip</nav><main><h1>Result Title</h1><p>Extract me.</p></main></body></html>",
        Some("cGl4ZWxz"),
        vec![serde_json::json!({
            "ok": true,
            "html": "<main><h1>Result Title</h1><p>Extract me.</p></main>"
        })],
    );

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", temp.path().join("home"))
        .args([
            "--envelope",
            "json",
            "interact",
            "current-tab",
            "--cdp-port",
            &port.to_string(),
            "--actions",
        ])
        .arg(&actions)
        .args([
            "--allow-actions",
            "--allow-private-content",
            "--capture-screenshot",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let envelope_text = String::from_utf8(output).unwrap();
    assert!(!envelope_text.contains("query-token"));
    assert!(!envelope_text.contains("secret-fragment"));
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["data"]["actions"]["succeeded"], 2);
    assert!(envelope["data"]["final_url"]
        .as_str()
        .unwrap()
        .contains("redacted"));
    let captures = envelope["data"]["artifacts"]["captures"]
        .as_array()
        .unwrap();
    let extracts = envelope["data"]["artifacts"]["extracts"]
        .as_array()
        .unwrap();
    assert_eq!(captures.len(), 2);
    assert_eq!(extracts.len(), 1);
    assert!(captures
        .iter()
        .all(|artifact| artifact["sensitive"] == true));
    assert_eq!(extracts[0]["sensitive"], true);

    let html_path = std::path::PathBuf::from(
        captures
            .iter()
            .find(|artifact| artifact["kind"] == "capture-html")
            .unwrap()["path"]
            .as_str()
            .unwrap(),
    );
    let screenshot_path = std::path::PathBuf::from(
        captures
            .iter()
            .find(|artifact| artifact["kind"] == "capture-screenshot")
            .unwrap()["path"]
            .as_str()
            .unwrap(),
    );
    let extract_path = std::path::PathBuf::from(extracts[0]["path"].as_str().unwrap());
    assert!(fs::read_to_string(&html_path)
        .unwrap()
        .contains("Result Title"));
    assert_eq!(fs::read(&screenshot_path).unwrap(), b"pixels");
    let extracted = fs::read_to_string(&extract_path).unwrap();
    assert!(extracted.contains("Result Title"));
    assert!(extracted.contains("Extract me."));

    let result_path = envelope["data"]["artifacts"]["actions_result"]
        .as_str()
        .unwrap();
    let result: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(result_path).unwrap()).unwrap();
    assert_eq!(
        result["actions"][0]["artifacts"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        result["actions"][1]["artifacts"].as_array().unwrap().len(),
        1
    );
    let metadata_path = envelope["data"]["artifacts"]["metadata"].as_str().unwrap();
    let metadata = fs::read_to_string(metadata_path).unwrap();
    assert!(!metadata.contains("query-token"));
    assert!(!metadata.contains("secret-fragment"));

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
    assert!(kinds.contains(&"capture"));
    assert!(kinds.contains(&"extract"));

    Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", temp.path().join("home"))
        .args(["--envelope", "json", "artifacts", "delete", run_id, "--yes"])
        .assert()
        .success();
    assert!(!html_path.exists());
    assert!(!screenshot_path.exists());
    assert!(!extract_path.exists());
    server.join().unwrap();
}

#[test]
fn interact_current_tab_capture_screenshot_selector_requires_unique_match() {
    let temp = tempfile::tempdir().unwrap();
    let actions = temp.path().join("actions.json");
    fs::write(
        &actions,
        r#"{
          "schema_version": "aget.actions.v1",
          "actions": [
            {"type": "capture", "name": "result", "selector": ".result", "screenshot": true}
          ]
        }"#,
    )
    .unwrap();
    let (port, server) = mock_current_tab_interact_cdp_server_with_page(
        "https://example.com/app",
        "<html><body><main class=\"result\">First</main><main class=\"result\">Second</main></body></html>",
        None,
        vec![serde_json::json!({
            "ok": false,
            "code": "selector_ambiguous",
            "message": "capture selector matched 2 elements"
        })],
    );

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", temp.path().join("home"))
        .args([
            "--envelope",
            "json",
            "interact",
            "current-tab",
            "--cdp-port",
            &port.to_string(),
            "--actions",
        ])
        .arg(&actions)
        .args([
            "--allow-actions",
            "--allow-private-content",
            "--capture-screenshot",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], "selector_ambiguous");
    assert_eq!(envelope["data"]["actions"]["failed_index"], 0);
    assert!(envelope["data"]["artifacts"]["captures"]
        .as_array()
        .map(Vec::is_empty)
        .unwrap_or(true));
    server.join().unwrap();
}

#[test]
fn interact_current_tab_extract_selector_requires_unique_match() {
    let temp = tempfile::tempdir().unwrap();
    let actions = temp.path().join("actions.json");
    fs::write(
        &actions,
        r#"{
          "schema_version": "aget.actions.v1",
          "actions": [
            {"type": "extract", "name": "main", "selector": ".result", "content_format": "markdown"}
          ]
        }"#,
    )
    .unwrap();
    let (port, server) = mock_current_tab_interact_cdp_server_with_page(
        "https://example.com/app",
        "<html><body><main class=\"result\">First</main><main class=\"result\">Second</main></body></html>",
        None,
        vec![serde_json::json!({
            "ok": false,
            "code": "selector_ambiguous",
            "message": "extract selector matched 2 elements"
        })],
    );

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", temp.path().join("home"))
        .args([
            "--envelope",
            "json",
            "interact",
            "current-tab",
            "--cdp-port",
            &port.to_string(),
            "--actions",
        ])
        .arg(&actions)
        .args(["--allow-actions", "--allow-private-content"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["error"]["code"], "selector_ambiguous");
    assert_eq!(envelope["data"]["actions"]["failed_index"], 0);
    server.join().unwrap();
}

#[test]
fn interact_current_tab_extract_selector_can_choose_first_match() {
    let temp = tempfile::tempdir().unwrap();
    let actions = temp.path().join("actions.json");
    fs::write(
        &actions,
        r#"{
          "schema_version": "aget.actions.v1",
          "actions": [
            {"type": "extract", "name": "result", "selector": ".result", "match": "first", "content_format": "markdown"}
          ]
        }"#,
    )
    .unwrap();
    let (port, server) = mock_current_tab_interact_cdp_server_with_page(
        "https://example.com/app",
        "<html><body><main class=\"result\">First</main><main class=\"result\">Second</main></body></html>",
        None,
        vec![serde_json::json!({
            "ok": true,
            "html": "<main class=\"result\"><h1>First</h1><p>Only first.</p></main>"
        })],
    );

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", temp.path().join("home"))
        .args([
            "--envelope",
            "json",
            "interact",
            "current-tab",
            "--cdp-port",
            &port.to_string(),
            "--actions",
        ])
        .arg(&actions)
        .args(["--allow-actions", "--allow-private-content"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let extract_path = envelope["data"]["artifacts"]["extracts"][0]["path"]
        .as_str()
        .unwrap();
    let extracted = fs::read_to_string(extract_path).unwrap();
    assert!(extracted.contains("First"));
    assert!(!extracted.contains("Second"));
    server.join().unwrap();
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
