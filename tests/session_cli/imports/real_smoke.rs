use assert_cmd::Command;

#[test]
#[ignore = "requires local Chrome/Chromium and an explicitly approved AGET_REAL_BROWSER_PROFILE"]
fn real_session_import_browser_chrome_profile() {
    let profile = match std::env::var("AGET_REAL_BROWSER_PROFILE") {
        Ok(profile) => profile,
        Err(_) => {
            eprintln!("set AGET_REAL_BROWSER_PROFILE to an approved Chrome profile name or path");
            return;
        }
    };
    let domain = std::env::var("AGET_REAL_BROWSER_DOMAIN").unwrap_or_else(|_| "example.com".into());
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    cmd.env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "browser",
            "--browser",
            "chrome",
            "--browser-profile",
            &profile,
            "--name",
            "real-browser-import",
            "--allow-domain",
            &domain,
        ])
        .assert()
        .success();
}
