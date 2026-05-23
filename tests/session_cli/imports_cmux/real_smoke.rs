use std::process::Command as StdCommand;

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;

use crate::support::session_cli::{
    crawl4ai_cookie_echo_server, loopback_cookie_echo_server, loopback_cookie_server,
    mock_backend_command,
};

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

    let store = aget::session::SessionStore::new(&aget_home).unwrap();
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
