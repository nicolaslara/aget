use assert_cmd::Command;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, chrome_import_state, success_data};

use super::success_assertions::{
    assert_backend_calls_cleaned, assert_inspect_redaction, assert_saved_session,
};

#[test]
fn session_import_chrome_saves_filtered_state_and_cleans_raw_file() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": chrome_import_state()}),
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
            "chrome-imported",
            "--allow-domain",
            "example.com",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.import.chrome");
    assert_eq!(json["source"], "chrome");
    assert_eq!(json["name"], "chrome-imported");
    assert_eq!(json["cookie_count"], 2);
    assert_eq!(json["origin_count"], 2);

    assert_saved_session(&aget_home);
    assert_inspect_redaction(&aget_home);
    assert_backend_calls_cleaned(&log_path);
}
