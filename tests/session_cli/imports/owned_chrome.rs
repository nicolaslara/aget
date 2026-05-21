use std::fs;

use assert_cmd::Command;

#[cfg(unix)]
#[test]
fn owned_session_import_chrome_classifies_profile_in_use_as_requires_user_action() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let profile = temp.path().join("chrome-profile");
    fs::create_dir_all(&profile).unwrap();
    let fake_chrome = temp.path().join("fake-chrome");
    fs::write(
        &fake_chrome,
        "#!/bin/sh\necho 'Opening in existing browser session.' >&2\nexit 0\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_chrome).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&fake_chrome, permissions).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CHROME_COMMAND", &fake_chrome)
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "chrome",
            "--chrome-profile",
            profile.to_str().unwrap(),
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
    assert_eq!(json["error"]["code"], "requires_user_action");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Opening in existing browser session"));
    assert!(!aget_home.join("sessions/imported.json").exists());
}

#[cfg(unix)]
#[test]
fn owned_session_import_chrome_reports_sandbox_startup_hint() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let profile = temp.path().join("chrome-profile");
    fs::create_dir_all(&profile).unwrap();
    let fake_chrome = temp.path().join("fake-chrome");
    fs::write(
        &fake_chrome,
        "#!/bin/sh\n\
         echo 'Failed to move to new namespace: sandbox error' >&2\n\
         exit 1\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_chrome).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&fake_chrome, permissions).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CHROME_COMMAND", &fake_chrome)
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "chrome",
            "--chrome-profile",
            profile.to_str().unwrap(),
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
    let message = json["error"]["message"].as_str().unwrap();
    assert!(message.contains("sandbox error"));
    assert!(message.contains("Chrome sandbox/namespace startup failure"));
    assert!(message.contains("AGET_CHROME_COMMAND"));
    assert!(!aget_home.join("sessions/imported.json").exists());
}
