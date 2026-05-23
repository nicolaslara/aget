use std::fs;

use assert_cmd::Command;
use serde_json::json;

use crate::support::get_cli::{metadata_files, mock_backend_command};

#[test]
fn get_timeout_returns_stable_error_and_error_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend =
        mock_backend_command(temp.path(), json!({"behavior": "sleep", "seconds": 3}));

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--timeout",
            "1",
            "--envelope",
            "json",
            "get",
            "https://example.com/slow",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["command"], "get");
    assert_eq!(json["error"]["code"], "timeout");

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["ok"], false);
    assert_eq!(metadata["extractor"], "crawl4ai");
    assert_eq!(metadata["error"]["code"], "timeout");
}

#[cfg(unix)]
#[test]
fn get_timeout_terminates_backend_descendants() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let marker = temp.path().join("descendant-survived.txt");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "spawn_descendant_and_sleep",
            "marker": marker,
            "seconds": 5
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--timeout",
            "1",
            "--envelope",
            "json",
            "get",
            "https://example.com/slow",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "timeout");
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert!(!marker.exists());
}
