use std::fs;
use std::path::PathBuf;
use std::process::Command as StdCommand;

use aget::session::SessionStore;
use aget::{
    complete_login_session, finish_login_session, start_login_session, LoginCompleteOptions,
    LoginFinishOptions, LoginStartOptions, SessionCookie, SessionSource,
};
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use support::session_cli::{
    agent_browser_tool, chrome_import_state, crawl4ai_cookie_echo_server, demo_session,
    loopback_cookie_echo_server, loopback_cookie_server, mock_backend_command, mock_cmux,
    named_session, nyt_state, provider_only_state, session_origin, success_data, success_envelope,
};
mod support;

#[test]
fn session_list_inspect_and_delete_use_aget_home() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let session = demo_session();
    store.save(&session).unwrap();

    let mut list = Command::cargo_bin("aget").unwrap();
    list.env("AGET_HOME", &aget_home)
        .args(["session", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("demo"));

    let mut inspect = Command::cargo_bin("aget").unwrap();
    inspect
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "demo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<redacted>"))
        .stdout(predicate::str::contains("secret-cookie").not());

    let mut inspect_secrets = Command::cargo_bin("aget").unwrap();
    inspect_secrets
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "demo", "--show-secrets"])
        .assert()
        .success()
        .stdout(predicate::str::contains("secret-cookie"));

    let mut delete = Command::cargo_bin("aget").unwrap();
    delete
        .env("AGET_HOME", &aget_home)
        .args(["session", "delete", "demo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted session demo"));

    assert!(store.list().unwrap().is_empty());
}

#[test]
fn session_list_json_has_stable_shape() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    store.save(&demo_session()).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "session", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.list");
    assert_eq!(json["sessions"], serde_json::json!(["demo"]));
}

#[test]
fn session_import_cmux_saves_filtered_cookies() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_cmux = mock_cmux(temp.path(), json!({"surface": "surface:1"}));

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CMUX_COMMAND", &fake_cmux)
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "cmux",
            "--surface",
            "surface:1",
            "--name",
            "imported",
            "--allow-domain",
            "example.com",
            "--allow-domain",
            "docs.example.com",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.import.cmux");
    assert_eq!(json["name"], "imported");
    assert_eq!(json["source"], "cmux");
    assert_eq!(json["cookie_count"], 4);

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("imported").unwrap();
    assert_eq!(
        session.source,
        SessionSource::Cmux {
            surface: "surface:1".to_string()
        }
    );
    assert!(session.sensitive);
    assert_eq!(
        session.allowed_cookie_domains,
        vec!["example.com".to_string(), "docs.example.com".to_string()]
    );
    assert!(session.allowed_storage_origins.is_empty());
    assert!(session.origins.is_empty());
    assert_eq!(session.cookies.len(), 4);
    assert!(session
        .cookies
        .iter()
        .all(|cookie| cookie.source_session.as_deref() == Some("imported")));
    assert!(session.cookies.iter().all(|cookie| cookie.http_only));
    assert!(session
        .cookies
        .iter()
        .all(|cookie| cookie.same_site.is_none()));
    assert!(session
        .cookies
        .iter()
        .any(|cookie| cookie.name == "sid" && cookie.domain == "example.com"));
    assert!(session
        .cookies
        .iter()
        .any(|cookie| cookie.name == "wide" && cookie.domain == ".example.com"));
    assert!(!session.cookies.iter().any(|cookie| cookie.name == "evil"));

    let mut inspect = Command::cargo_bin("aget").unwrap();
    inspect
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "imported"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<redacted>"))
        .stdout(predicate::str::contains("allowed-secret").not())
        .stdout(predicate::str::contains("suffix-secret").not())
        .stdout(predicate::str::contains("blocked-secret").not());
}

#[test]
fn session_import_cmux_missing_backend_returns_backend_unavailable() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CMUX_COMMAND", "definitely_missing_aget_cmux")
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "cmux",
            "--surface",
            "surface:1",
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
    assert_eq!(json["command"], "session.import.cmux");
    assert_eq!(json["error"]["code"], "backend_unavailable");
}

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

#[test]
fn session_login_start_opens_aget_owned_browser_and_records_pending_flow() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let target_url = "https://www.nytimes.com/article";
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));
    let expected_profile = aget_home.join("tmp/agent-browser/aget-news");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "news",
            "--url",
            target_url,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope = success_envelope(&output, "session.login.start");
    assert!(envelope["warnings"][0]
        .as_str()
        .unwrap()
        .contains("prefer signing in with your real browser"));
    let json = envelope["data"].clone();
    assert_eq!(json["state"], "login_started");
    assert!(json.get("site").is_none());
    assert_eq!(json["name"], "news");
    assert_eq!(
        PathBuf::from(json["profile"].as_str().unwrap()),
        expected_profile
    );
    assert_eq!(json["url"], target_url);
    assert_eq!(
        json["allowed_domains"],
        serde_json::json!(["nytimes.com", "www.nytimes.com"])
    );
    assert_eq!(
        json["next_command"],
        serde_json::json!(["aget", "session", "login", "finish", "news"])
    );
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(&format!(
        r#""--profile", "{}", "--session", "aget-login-news", "open""#,
        expected_profile.display()
    )));
    assert!(log.contains(target_url));
    assert!(aget_home.join("tmp/login-news.json").exists());
    assert!(expected_profile.parent().unwrap().exists());
}

#[test]
fn session_login_start_rejects_http_url_before_agent_browser() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "news",
            "--url",
            "http://www.nytimes.com/login",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "session.login.start");
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("login URL must use https"));
    assert!(!log_path.exists() || fs::read_to_string(&log_path).unwrap().is_empty());
    assert!(!aget_home.join("tmp/login-news.json").exists());
}

#[test]
fn session_login_start_uses_exact_non_www_url_host_scope() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "docs",
            "--url",
            "https://docs.example.com/login",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.login.start");
    assert_eq!(json["name"], "docs");
    assert_eq!(
        json["allowed_domains"],
        serde_json::json!(["docs.example.com"])
    );
    assert!(aget_home.join("tmp/login-docs.json").exists());
}

#[test]
fn session_login_start_rejects_duplicate_pending_flow_without_overwriting() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

    let first_url = "https://www.nytimes.com/article";
    let second_url = "https://www.nytimes.com/login";

    let mut first = Command::cargo_bin("aget").unwrap();
    first
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "news",
            "--url",
            first_url,
            "--profile",
            "aget-hi",
        ])
        .assert()
        .success();

    let pending_path = aget_home.join("tmp/login-news.json");
    let original_pending = fs::read_to_string(&pending_path).unwrap();

    let mut second = Command::cargo_bin("aget").unwrap();
    let output = second
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "news",
            "--url",
            second_url,
            "--profile",
            "aget-hi-2",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("pending login flow named 'news' already exists"));
    assert_eq!(fs::read_to_string(&pending_path).unwrap(), original_pending);
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#""--profile", "aget-hi", "--session", "aget-login-news", "open""#));
    assert!(!log.contains(r#"aget-hi-2"#));
    assert!(!log.contains(r#"["--session", "aget-login-news", "close"]"#));
}

#[test]
fn session_login_finish_saves_only_url_scoped_state_and_cleans_temp_files() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": nyt_state("nyt_session", "token")}),
    );

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "news",
            "--url",
            "https://www.nytimes.com/article",
        ])
        .assert()
        .success();

    let mut finish = Command::cargo_bin("aget").unwrap();
    let output = finish
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args(["--envelope", "json", "session", "login", "finish", "news"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.login.finish");
    assert_eq!(json["state"], "login_finished");
    assert_eq!(json["name"], "news");
    assert_eq!(json["cookie_count"], 1);
    assert_eq!(json["origin_count"], 1);

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("news").unwrap();
    assert_eq!(
        session.source,
        SessionSource::AgentBrowser {
            session: "aget-login-news".to_string()
        }
    );
    assert_eq!(
        session.allowed_cookie_domains,
        vec!["nytimes.com".to_string(), "www.nytimes.com".to_string()]
    );
    assert_eq!(session.cookies.len(), 1);
    assert_eq!(session.cookies[0].name, "nyt_session");
    assert_eq!(session.origins.len(), 1);
    assert_eq!(session.origins[0].origin, "https://www.nytimes.com");
    assert!(!session
        .cookies
        .iter()
        .any(|cookie| cookie.domain.contains("google")));
    assert!(!session
        .origins
        .iter()
        .any(|origin| origin.origin.contains("google")));
    assert!(!aget_home.join("tmp/login-news.json").exists());
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#""state", "save""#));
    assert!(log.contains(r#""close""#));
    let raw_state_path = log
        .lines()
        .find_map(|line| line.strip_prefix("STATE_PATH="))
        .map(PathBuf::from)
        .unwrap();
    assert!(!raw_state_path.exists());
}

#[test]
fn session_login_finish_merges_new_scope_into_existing_bucket() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let store = SessionStore::new(&aget_home).unwrap();
    let mut existing = named_session("news", "nyt_old", "old-nyt-secret", ".nytimes.com");
    existing.allowed_cookie_domains.push("ft.com".to_string());
    existing.cookies.push(SessionCookie {
        name: "ft_session".to_string(),
        value: "ft-secret".to_string(),
        domain: "ft.com".to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some("Lax".to_string()),
        source_session: None,
    });
    existing.allowed_storage_origins = vec![
        "https://www.nytimes.com".to_string(),
        "https://www.ft.com".to_string(),
    ];
    existing.origins.push(session_origin(
        "https://www.nytimes.com",
        "old-token",
        "old-storage",
    ));
    existing.origins.push(session_origin(
        "https://www.ft.com",
        "ft-token",
        "ft-storage",
    ));
    store.save(&existing).unwrap();

    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": nyt_state("nyt_new", "new-token")}),
    );

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "news",
            "--url",
            "https://www.nytimes.com/article",
        ])
        .assert()
        .success();

    let mut finish = Command::cargo_bin("aget").unwrap();
    finish
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args(["session", "login", "finish", "news"])
        .assert()
        .success();

    let merged = store.load("news").unwrap();
    assert!(merged.cookies.iter().any(|cookie| cookie.name == "nyt_new"));
    assert!(merged
        .cookies
        .iter()
        .any(|cookie| cookie.name == "ft_session"));
    assert!(!merged.cookies.iter().any(|cookie| cookie.name == "nyt_old"));
    assert!(merged.origins.iter().any(|origin| {
        origin.origin == "https://www.nytimes.com"
            && origin
                .local_storage
                .iter()
                .any(|entry| entry.name == "new-token")
    }));
    assert!(merged
        .origins
        .iter()
        .any(|origin| origin.origin == "https://www.ft.com"));
    assert!(!merged.origins.iter().any(|origin| {
        origin.origin == "https://www.nytimes.com"
            && origin
                .local_storage
                .iter()
                .any(|entry| entry.name == "old-token")
    }));
    assert_eq!(
        merged.allowed_cookie_domains,
        vec![
            "ft.com".to_string(),
            "nytimes.com".to_string(),
            "www.nytimes.com".to_string()
        ]
    );
}

#[test]
fn session_login_finish_reports_close_failure_and_keeps_pending_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({
            "behavior": "close_failure",
            "state": nyt_state("nyt_session", "token"),
            "close_stderr": "close failed"
        }),
    );

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "news",
            "--url",
            "https://www.nytimes.com/article",
        ])
        .assert()
        .success();

    let mut finish = Command::cargo_bin("aget").unwrap();
    let output = finish
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args(["--envelope", "json", "session", "login", "finish", "news"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("close failed"));
    assert!(aget_home.join("tmp/login-news.json").exists());
    assert!(SessionStore::new(&aget_home).unwrap().load("news").is_err());
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#"["--session", "aget-login-news", "close"]"#));
}

#[test]
fn finish_login_session_leaves_pending_until_complete_login_session_runs() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let tmp_dir = aget_home.join("tmp");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": nyt_state("nyt_session", "token")}),
    );
    unsafe {
        std::env::set_var("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser);
        std::env::set_var("AGET_FAKE_AGENT_BROWSER_LOG", &log_path);
    }

    let start_result = start_login_session(LoginStartOptions {
        name: "hi".to_string(),
        profile: None,
        url: "https://www.nytimes.com/article".to_string(),
        tmp_dir: tmp_dir.clone(),
    })
    .unwrap();
    assert!(tmp_dir.join("login-hi.json").exists());
    assert_eq!(
        PathBuf::from(&start_result.pending.profile),
        tmp_dir.join("agent-browser/aget-hi")
    );
    let profile_dir = PathBuf::from(&start_result.pending.profile);
    fs::create_dir_all(&profile_dir).unwrap();

    let finish_result = finish_login_session(LoginFinishOptions {
        name: "hi".to_string(),
        tmp_dir: tmp_dir.clone(),
    })
    .unwrap();
    assert_eq!(finish_result.pending, start_result.pending);
    assert_eq!(finish_result.session.cookies[0].expires, Some(1812619153));
    assert!(tmp_dir.join("login-hi.json").exists());

    complete_login_session(LoginCompleteOptions {
        pending: finish_result.pending,
        tmp_dir,
    })
    .unwrap();

    assert!(!aget_home.join("tmp/login-hi.json").exists());
    assert!(!profile_dir.exists());
}

#[test]
fn session_login_cancel_closes_only_pending_agent_browser_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(temp.path(), &log_path, json!({}));

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "hellointerview",
            "--url",
            "https://www.hellointerview.com/login",
        ])
        .assert()
        .success();

    let mut cancel = Command::cargo_bin("aget").unwrap();
    let output = cancel
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "cancel",
            "hellointerview",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.login.cancel");
    assert_eq!(json["state"], "login_cancelled");
    assert_eq!(json["name"], "hellointerview");
    assert_eq!(json["agent_session"], "aget-login-hellointerview");
    assert!(!aget_home.join("tmp/login-hellointerview.json").exists());
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains(r#""open""#));
    assert!(log.contains(r#"["--session", "aget-login-hellointerview", "close"]"#));
}

#[test]
fn session_login_cancel_cleans_profile_and_pending_when_close_fails() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"behavior": "close_failure", "close_stderr": "close failed", "close_exit_code": 2}),
    );

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "news",
            "--url",
            "https://www.nytimes.com/login",
        ])
        .assert()
        .success();
    let profile = aget_home.join("tmp/agent-browser/aget-news");
    fs::create_dir_all(&profile).unwrap();

    let mut cancel = Command::cargo_bin("aget").unwrap();
    let output = cancel
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args(["--envelope", "json", "session", "login", "cancel", "news"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["command"], "session.login.cancel");
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert!(!aget_home.join("tmp/login-news.json").exists());
    assert!(!profile.exists());
}

#[test]
fn session_login_finish_rejects_provider_only_state_without_saving_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": provider_only_state()}),
    );
    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "session",
            "login",
            "start",
            "hellointerview",
            "--url",
            "https://www.hellointerview.com/login",
        ])
        .assert()
        .success();

    let mut finish = Command::cargo_bin("aget").unwrap();
    let output = finish
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .env("AGET_FAKE_AGENT_BROWSER_LOG", &log_path)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "finish",
            "hellointerview",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "requires_user_action");
    assert!(SessionStore::new(&aget_home)
        .unwrap()
        .load("hellointerview")
        .is_err());
    assert!(aget_home.join("tmp/login-hellointerview.json").exists());
}

#[test]
#[ignore = "requires local agent-browser, Crawl4AI setup, and manual authorized HelloInterview login"]
fn real_hellointerview_login_flow_fetches_paywalled_markdown() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let target = std::env::var("AGET_REAL_HELLOINTERVIEW_URL").unwrap_or_else(|_| {
        "https://www.hellointerview.com/learn/behavioral/course/select-choosing-responses-strategically".to_string()
    });

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "hellointerview",
            "--url",
            &target,
        ])
        .assert()
        .success();

    eprintln!(
        "Complete the HelloInterview/Google login in the opened browser, then press Enter here."
    );
    let mut confirmation = String::new();
    std::io::stdin().read_line(&mut confirmation).unwrap();

    let mut finish = Command::cargo_bin("aget").unwrap();
    finish
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "finish",
            "hellointerview",
        ])
        .assert()
        .success();

    let mut get = Command::cargo_bin("aget").unwrap();
    let output = get
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            &target,
            "--session",
            "hellointerview",
            "--content-format",
            "markdown",
            "--timeout",
            "90",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    let content = json["content"].as_str().unwrap();
    assert!(!content.contains("Purchase Premium to Keep Reading"));
    assert!(!content.contains("Premium users can view this video once signed in"));
    assert!(content.len() > 500);
}

#[test]
fn session_compose_persists_composed_session_with_provenance_and_preserves_sources() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let mut provider = named_session(
        "provider",
        "oauth",
        "provider-secret",
        "accounts.example.com",
    );
    provider.allowed_storage_origins = vec!["https://accounts.example.com".to_string()];
    provider.origins.push(session_origin(
        "https://accounts.example.com",
        "provider-token",
        "provider-storage-secret",
    ));
    let app = named_session("app", "appsid", "app-secret", "app.example.com");
    let original_provider = provider.clone();
    let original_app = app.clone();
    store.save(&provider).unwrap();
    store.save(&app).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.compose");
    assert_eq!(json["name"], "combined");
    assert_eq!(
        json["source_sessions"],
        serde_json::json!(["provider", "app"])
    );
    assert_eq!(json["cookie_count"], 2);
    assert_eq!(json["origin_count"], 1);

    let combined = store.load("combined").unwrap();
    assert_eq!(
        combined.source,
        SessionSource::Composed {
            sessions: vec!["provider".to_string(), "app".to_string()]
        }
    );
    assert_eq!(
        combined.allowed_cookie_domains,
        vec![
            "accounts.example.com".to_string(),
            "app.example.com".to_string()
        ]
    );
    assert!(combined.cookies.iter().any(|cookie| {
        cookie.name == "oauth" && cookie.source_session.as_deref() == Some("provider")
    }));
    assert!(combined.cookies.iter().any(|cookie| {
        cookie.name == "appsid" && cookie.source_session.as_deref() == Some("app")
    }));
    assert_eq!(
        combined.origins[0].source_session.as_deref(),
        Some("provider")
    );
    assert_eq!(store.load("provider").unwrap(), original_provider);
    assert_eq!(store.load("app").unwrap(), original_app);
}

#[test]
fn session_compose_rejects_source_name_target_without_mutating_sources() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let provider = named_session(
        "provider",
        "oauth",
        "provider-secret",
        "accounts.example.com",
    );
    let app = named_session("app", "appsid", "app-secret", "app.example.com");
    store.save(&provider).unwrap();
    store.save(&app).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "compose",
            "provider",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("must not match a source session"));
    assert_eq!(store.load("provider").unwrap(), provider);
    assert_eq!(store.load("app").unwrap(), app);
}

#[test]
fn session_compose_rejects_existing_target_without_mutating_existing_session() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let provider = named_session(
        "provider",
        "oauth",
        "provider-secret",
        "accounts.example.com",
    );
    let app = named_session("app", "appsid", "app-secret", "app.example.com");
    let existing = named_session("combined", "existing", "existing-secret", "old.example.com");
    store.save(&provider).unwrap();
    store.save(&app).unwrap();
    store.save(&existing).unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("already exists"));
    assert_eq!(store.load("combined").unwrap(), existing);
    assert_eq!(store.load("provider").unwrap(), provider);
    assert_eq!(store.load("app").unwrap(), app);
}

#[test]
fn session_compose_inspect_reports_cookie_source_session_and_redacts_values() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    store
        .save(&named_session(
            "provider",
            "oauth",
            "provider-secret",
            "accounts.example.com",
        ))
        .unwrap();
    store
        .save(&named_session(
            "app",
            "appsid",
            "app-secret",
            "app.example.com",
        ))
        .unwrap();

    let mut compose = Command::cargo_bin("aget").unwrap();
    compose
        .env("AGET_HOME", &aget_home)
        .args([
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success();

    let mut inspect = Command::cargo_bin("aget").unwrap();
    let inspect_output = inspect
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "session", "inspect", "combined"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect_json = success_data(&inspect_output, "session.inspect");
    assert_eq!(inspect_json["cookies"][0]["value"], "<redacted>");
    assert!(inspect_json["cookies"]
        .as_array()
        .unwrap()
        .iter()
        .any(|cookie| { cookie["name"] == "oauth" && cookie["source_session"] == "provider" }));
    assert!(!String::from_utf8_lossy(&inspect_output).contains("provider-secret"));
    assert!(!String::from_utf8_lossy(&inspect_output).contains("app-secret"));
}

#[test]
fn session_inspect_plain_reports_cookie_and_origin_provenance_with_redacted_values() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    let mut provider = named_session(
        "provider",
        "oauth",
        "provider-secret",
        "accounts.example.com",
    );
    provider.allowed_storage_origins = vec!["https://accounts.example.com".to_string()];
    provider.origins.push(session_origin(
        "https://accounts.example.com",
        "provider-token",
        "provider-storage-secret",
    ));
    store.save(&provider).unwrap();
    store
        .save(&named_session(
            "app",
            "appsid",
            "app-secret",
            "app.example.com",
        ))
        .unwrap();

    let mut compose = Command::cargo_bin("aget").unwrap();
    compose
        .env("AGET_HOME", &aget_home)
        .args([
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success();

    let mut inspect = Command::cargo_bin("aget").unwrap();
    inspect
        .env("AGET_HOME", &aget_home)
        .args(["session", "inspect", "combined"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "accounts.example.com oauth=<redacted> source=provider",
        ))
        .stdout(predicate::str::contains(
            "https://accounts.example.com source=provider",
        ))
        .stdout(predicate::str::contains(
            "localStorage provider-token=<redacted>",
        ))
        .stdout(predicate::str::contains("provider-secret").not())
        .stdout(predicate::str::contains("provider-storage-secret").not());
}

#[test]
fn session_compose_rejects_conflicts_with_redacted_error_and_does_not_save() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let store = SessionStore::new(&aget_home).unwrap();
    store
        .save(&named_session(
            "first",
            "sid",
            "first-secret",
            "example.com",
        ))
        .unwrap();
    store
        .save(&named_session(
            "second",
            "sid",
            "second-secret",
            "example.com",
        ))
        .unwrap();

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "compose",
            "combined",
            "--session",
            "first",
            "--session",
            "second",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "session_conflict");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("conflicting cookie 'sid'"));
    assert!(!String::from_utf8_lossy(&output).contains("first-secret"));
    assert!(!String::from_utf8_lossy(&output).contains("second-secret"));
    assert!(store.load("combined").is_err());
}

#[test]
#[ignore = "requires a running cmux browser surface named by AGET_REAL_CMUX_SURFACE"]
fn real_cmux_imports_loopback_cookie() {
    let surface = match std::env::var("AGET_REAL_CMUX_SURFACE") {
        Ok(surface) => surface,
        Err(_) => {
            eprintln!("set AGET_REAL_CMUX_SURFACE to a disposable cmux browser surface");
            return;
        }
    };
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let (url, server) = loopback_cookie_server();

    let goto = StdCommand::new("cmux")
        .args(["browser", "--surface", &surface, "goto", &url])
        .status()
        .expect("cmux must be installed and runnable");
    assert!(goto.success(), "cmux goto failed with {goto}");
    server.join().unwrap();

    let mut import = Command::cargo_bin("aget").unwrap();
    import
        .env("AGET_HOME", &aget_home)
        .args([
            "session",
            "import",
            "cmux",
            "--surface",
            &surface,
            "--name",
            "cmux-loopback",
            "--allow-domain",
            "127.0.0.1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Imported cmux session cmux-loopback",
        ));

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("cmux-loopback").unwrap();
    assert!(session.cookies.iter().any(|cookie| {
        cookie.name == "aget_cmux_e2e"
            && cookie.value == "loopback-secret"
            && cookie.domain == "127.0.0.1"
    }));

    let (echo_url, echo_server) = loopback_cookie_echo_server();
    let replay_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "http_fetch",
            "content_prefix": "# cmux replay ok\n\n",
            "expect_state_cookies": [["aget_cmux_e2e", "loopback-secret"]]
        }),
    );

    let mut replay = Command::cargo_bin("aget").unwrap();
    replay
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &replay_backend)
        .args(["get", &echo_url, "--session", "cmux-loopback"])
        .assert()
        .success()
        .stdout(predicate::str::contains("aget_cmux_e2e=loopback-secret"));
    echo_server.join().unwrap();
}

#[test]
#[ignore = "requires a running cmux browser surface and local Crawl4AI setup"]
fn real_cmux_import_replays_loopback_cookie_through_crawl4ai() {
    let surface = match std::env::var("AGET_REAL_CMUX_SURFACE") {
        Ok(surface) => surface,
        Err(_) => {
            eprintln!("set AGET_REAL_CMUX_SURFACE to a disposable cmux browser surface");
            return;
        }
    };
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let (url, server) = loopback_cookie_server();

    let goto = StdCommand::new("cmux")
        .args(["browser", "--surface", &surface, "goto", &url])
        .status()
        .expect("cmux must be installed and runnable");
    assert!(goto.success(), "cmux goto failed with {goto}");
    server.join().unwrap();

    let mut import = Command::cargo_bin("aget").unwrap();
    import
        .env("AGET_HOME", &aget_home)
        .args([
            "session",
            "import",
            "cmux",
            "--surface",
            &surface,
            "--name",
            "cmux-loopback",
            "--allow-domain",
            "127.0.0.1",
        ])
        .assert()
        .success();

    let (echo_url, echo_server, cookies) = crawl4ai_cookie_echo_server();
    let mut replay = Command::cargo_bin("aget").unwrap();
    replay
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            &echo_url,
            "--session",
            "cmux-loopback",
            "--timeout",
            "60",
        ])
        .assert()
        .success();
    echo_server.join().unwrap();

    assert!(cookies
        .try_iter()
        .any(|cookie| cookie.contains("aget_cmux_e2e=loopback-secret")));
}
