use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::agent_browser_tool;

#[test]
fn session_import_chrome_closes_and_cleans_raw_state_on_malformed_state() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"behavior": "malformed_state"}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "chrome",
            "--chrome-profile",
            "Default",
            "--name",
            "imported",
            "--allow-domain",
            "example.com",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "extraction_failed");
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#""open", "about:blank""#));
    assert!(log.contains(r#""state", "save""#));
    assert!(log.contains(r#""close""#));
    let raw_state_path = log
        .lines()
        .find_map(|line| line.strip_prefix("STATE_PATH="))
        .map(PathBuf::from)
        .unwrap();
    assert!(!raw_state_path.exists());
}
