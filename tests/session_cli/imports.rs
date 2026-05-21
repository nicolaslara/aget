use std::fs;
use std::path::PathBuf;

use aget::session::SessionStore;
use aget::SessionSource;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, chrome_import_state, success_data};

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

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("chrome-imported").unwrap();
    assert_eq!(
        session.source,
        SessionSource::ChromeProfile {
            profile: "Default".to_string()
        }
    );
    assert!(session.sensitive);
    assert_eq!(
        session.allowed_cookie_domains,
        vec!["example.com".to_string()]
    );
    assert_eq!(
        session.allowed_storage_origins,
        vec![
            "https://docs.example.com:443".to_string(),
            "https://example.com".to_string(),
        ]
    );
    assert_eq!(session.cookies.len(), 2);
    assert!(session.cookies.iter().all(|cookie| cookie
        .source_session
        .as_deref()
        .is_some_and(|source| source.starts_with("aget-import-"))));
    assert!(session.cookies.iter().any(|cookie| cookie.name == "sid"));
    assert!(session.cookies.iter().any(|cookie| cookie.name == "sub"));
    assert!(!session.cookies.iter().any(|cookie| cookie.name == "evil"));
    assert_eq!(
        session
            .cookies
            .iter()
            .find(|cookie| cookie.name == "sid")
            .unwrap()
            .expires,
        Some(1812619153)
    );
    assert_eq!(session.origins.len(), 2);
    assert!(session
        .origins
        .iter()
        .any(|origin| origin.origin == "https://example.com"
            && origin
                .local_storage
                .iter()
                .any(|entry| entry.name == "token")));
    assert!(session
        .origins
        .iter()
        .any(|origin| origin.origin == "https://example.com"
            && origin
                .session_storage
                .iter()
                .any(|entry| entry.name == "session-token")));
    assert!(!session
        .origins
        .iter()
        .any(|origin| origin.origin == "https://example.com.evil"));

    let mut inspect = Command::cargo_bin("aget").unwrap();
    let inspect_output = inspect
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "inspect",
            "chrome-imported",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect_json = success_data(&inspect_output, "session.inspect");
    let inspect_origin = inspect_json["origins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|origin| origin["origin"] == "https://example.com")
        .unwrap();
    assert_eq!(inspect_origin["local_storage"][0]["value"], "<redacted>");
    assert_eq!(inspect_origin["session_storage"][0]["value"], "<redacted>");
    assert!(!String::from_utf8_lossy(&inspect_output).contains("allowed-storage"));
    assert!(!String::from_utf8_lossy(&inspect_output).contains("session-only"));

    let mut inspect_secrets = Command::cargo_bin("aget").unwrap();
    inspect_secrets
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "inspect",
            "chrome-imported",
            "--show-secrets",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("allowed-storage"))
        .stdout(predicate::str::contains("session-only"));

    let log = fs::read_to_string(&log_path).unwrap();
    let calls = log
        .lines()
        .filter(|line| line.starts_with('['))
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 3);
    assert!(calls[0].contains(r#""--profile", "Default", "--session", "#));
    assert!(calls[1].contains(r#""state", "save""#));
    assert!(calls[2].contains(r#""close""#));
    let raw_state_path = log
        .lines()
        .find_map(|line| line.strip_prefix("STATE_PATH="))
        .map(PathBuf::from)
        .unwrap();
    assert!(!raw_state_path.exists());
}

#[test]
fn session_import_browser_chrome_saves_filtered_state() {
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
            "browser",
            "--browser",
            "chrome",
            "--browser-profile",
            "Default",
            "--name",
            "browser-imported",
            "--allow-domain",
            "example.com",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.import.browser");
    assert_eq!(json["source"], "browser_profile");
    assert_eq!(json["browser"], "chrome");
    assert_eq!(json["name"], "browser-imported");
    assert_eq!(json["cookie_count"], 2);
    assert_eq!(json["origin_count"], 2);

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("browser-imported").unwrap();
    assert_eq!(
        session.source,
        SessionSource::ChromeProfile {
            profile: "Default".to_string()
        }
    );
    assert!(session.sensitive);
    assert_eq!(session.cookies.len(), 2);
    assert!(!session.cookies.iter().any(|cookie| cookie.name == "evil"));
}

#[test]
fn session_import_browser_rejects_unsupported_browser_without_importing() {
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
            "browser",
            "--browser",
            "firefox",
            "--profile-path",
            "/tmp/firefox-profile",
            "--name",
            "firefox-imported",
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
    assert_eq!(json["command"], "session.import.browser");
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("does not support 'firefox' yet"));
    assert!(!log_path.exists() || fs::read_to_string(&log_path).unwrap().is_empty());
    assert!(!aget_home.join("sessions/firefox-imported.json").exists());
}

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

#[test]
fn session_import_chrome_requires_user_action_for_profile_lock() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser =
        agent_browser_tool(temp.path(), &log_path, json!({"behavior": "profile_lock"}));

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
    assert_eq!(json["error"]["code"], "requires_user_action");
}

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
