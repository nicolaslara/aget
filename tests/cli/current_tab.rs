use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;

use super::support::{mock_current_tab_cdp_server, mock_current_tab_cdp_server_with_screenshot};

#[test]
fn current_tab_requires_private_content_consent() {
    let temp = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .args(["--envelope", "json", "current-tab", "--cdp-port", "9222"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["command"], "current-tab");
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("private content"));
}

#[test]
fn current_tab_renders_mock_cdp_page_without_browser_state() {
    let temp = tempfile::tempdir().unwrap();
    let (port, server) = mock_current_tab_cdp_server(
        "https://example.com/current",
        "<html><body><nav>skip</nav><main><h1>Tab Title</h1><p>Private tab body.</p></main></body></html>",
    );
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .args([
            "--envelope",
            "json",
            "current-tab",
            "--cdp-port",
            &port.to_string(),
            "--allow-private-content",
            "--inline-content",
            "always",
            "--selector",
            "main",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "current-tab");
    assert_eq!(json["data"]["url"], "current-tab");
    assert_eq!(json["data"]["final_url"], "https://example.com/current");
    assert_eq!(json["data"]["extractor"], "aget-owned-current-tab");
    assert_eq!(json["data"]["sensitive"], true);
    assert!(json["data"]["content"]
        .as_str()
        .unwrap()
        .contains("Tab Title"));
    assert!(json["data"]["content"]
        .as_str()
        .unwrap()
        .contains("Private tab body."));
    assert!(json["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning.as_str().unwrap().contains("private")));
    server.join().unwrap();
}

#[test]
fn current_tab_writes_opt_in_debug_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let (port, server) = mock_current_tab_cdp_server_with_screenshot(
        "https://example.com/current?token=secret#frag",
        "<html><body><main><h1>Debug Tab</h1></main></body></html>",
        Some("cGl4ZWxz"),
    );
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .args([
            "--envelope",
            "json",
            "current-tab",
            "--cdp-port",
            &port.to_string(),
            "--allow-private-content",
            "--capture-screenshot",
            "--capture-trace",
            "--inline-content",
            "never",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    let data = &json["data"];
    assert!(data.get("content").is_none());
    let screenshot_path = PathBuf::from(
        data["artifacts"]["debug"]["screenshot"]["path"]
            .as_str()
            .unwrap(),
    );
    let trace_path = PathBuf::from(
        data["artifacts"]["debug"]["trace"]["path"]
            .as_str()
            .unwrap(),
    );
    assert_eq!(fs::read(&screenshot_path).unwrap(), b"pixels");
    let trace: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&trace_path).unwrap()).unwrap();
    assert_eq!(trace["schema_version"], "aget.debug_trace.v1");
    assert_eq!(trace["ok"], true);
    let final_url = trace["final_url"].as_str().unwrap();
    assert!(final_url.contains("redacted"));
    assert!(!final_url.contains("secret"));
    assert!(!final_url.contains("frag"));
    assert_eq!(trace["capture"]["screenshot_captured"], true);
    assert!(trace.get("content").is_none());

    let metadata_path = PathBuf::from(data["artifacts"]["metadata"].as_str().unwrap());
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(
        metadata["artifacts"]["debug"]["screenshot"]["path"],
        screenshot_path.to_string_lossy().as_ref()
    );
    assert_eq!(
        metadata["artifacts"]["debug"]["screenshot"]["sensitive"],
        true
    );
    server.join().unwrap();
}
