use assert_cmd::Command;

use super::support::mock_current_tab_cdp_server;

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
fn current_tab_renders_mock_cdp_page_without_agent_browser() {
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
fn current_tab_uses_owned_backend_when_agent_browser_compat_env_is_set() {
    let temp = tempfile::tempdir().unwrap();
    let (port, server) = mock_current_tab_cdp_server(
        "https://example.com/current-env",
        "<html><body><main><h1>Env Tab</h1><p>Owned path wins.</p></main></body></html>",
    );
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .env(
            "AGET_AGENT_BROWSER_COMMAND",
            "/definitely/not-agent-browser",
        )
        .args([
            "--envelope",
            "json",
            "current-tab",
            "--cdp-port",
            &port.to_string(),
            "--allow-private-content",
            "--inline-content",
            "always",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "current-tab");
    assert_eq!(json["data"]["final_url"], "https://example.com/current-env");
    assert_eq!(json["data"]["content"], "Env Tab Owned path wins.");
    server.join().unwrap();
}
