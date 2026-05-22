use assert_cmd::Command;

#[test]
fn session_import_chrome_missing_backend_returns_backend_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env(
            "AGET_AGENT_BROWSER_COMMAND",
            "definitely_missing_aget_agent_browser",
        )
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
    assert_eq!(json["error"]["code"], "backend_unavailable");
}
