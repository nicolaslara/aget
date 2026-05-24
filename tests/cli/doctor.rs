use std::fs;

use assert_cmd::Command;

#[test]
fn doctor_json_reports_optional_dependencies_without_failing_static_fetch() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CHROME_COMMAND", temp.path().join("missing-chrome"))
        .env("AGET_CMUX_COMMAND", temp.path().join("missing-cmux"))
        .args(["--envelope", "json", "doctor"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "doctor");
    assert_eq!(json["schema_version"], aget::ENVELOPE_SCHEMA_VERSION);
    assert!(json["data"]["summary"]["ok"].as_u64().unwrap() > 0);
    assert!(json["data"]["summary"]["warn"].as_u64().unwrap() >= 2);
    assert_eq!(json["data"]["summary"]["fail"], 0);

    let checks = json["data"]["checks"].as_array().unwrap();
    assert!(checks
        .iter()
        .any(|check| check["id"] == "chrome.command" && check["status"] == "warn"));
    assert!(checks
        .iter()
        .any(|check| check["id"] == "cmux.command" && check["status"] == "warn"));
    assert!(checks
        .iter()
        .any(|check| check["id"] == "store.layout" && check["status"] == "ok"));

    let ids = checks
        .iter()
        .map(|check| check["id"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), checks.len());
    assert!(!String::from_utf8(output)
        .unwrap()
        .contains("session-secret"));
}

#[cfg(unix)]
#[test]
fn doctor_warns_on_loose_session_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    fs::create_dir_all(aget_home.join("sessions")).unwrap();
    fs::write(aget_home.join("sessions/demo.json"), "{}").unwrap();
    fs::set_permissions(
        aget_home.join("sessions/demo.json"),
        fs::Permissions::from_mode(0o644),
    )
    .unwrap();

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "doctor", "--check", "store"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    let checks = json["data"]["checks"].as_array().unwrap();
    assert!(checks
        .iter()
        .any(|check| check["id"] == "store.permissions" && check["status"] == "warn"));
}
