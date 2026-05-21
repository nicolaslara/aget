use std::process::Command as StdCommand;

use aget::session::SessionStore;
use aget::SessionSource;
use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;

use crate::support::session_cli::{
    crawl4ai_cookie_echo_server, loopback_cookie_echo_server, loopback_cookie_server,
    mock_backend_command, mock_cmux, success_data,
};

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
