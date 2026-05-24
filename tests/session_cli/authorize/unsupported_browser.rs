use assert_cmd::Command;

#[test]
fn session_authorize_rejects_unsupported_browser_without_importing() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "authorize",
            "news",
            "--url",
            "https://example.com/private",
            "--browser",
            "firefox",
            "--browser-profile",
            "/tmp/firefox-profile",
            "--allow-domain",
            "example.com",
            "--must-contain",
            "Welcome",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["command"], "session.authorize");
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("does not support 'firefox' yet"));
    assert!(!aget_home.join("sessions/news.json").exists());
}
