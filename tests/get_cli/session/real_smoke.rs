use assert_cmd::Command;

use crate::support::get_cli::{cookie_echo_server, save_cookie_session, success_data};

#[test]
#[ignore = "requires local Crawl4AI and Playwright browser setup"]
fn real_crawl4ai_replays_named_session_cookie() {
    let temp = tempfile::tempdir().unwrap();
    let empty_home = temp.path().join("empty-home");
    let (empty_url, empty_server, empty_cookie) = cookie_echo_server("/cookie-empty");

    let mut empty_cmd = Command::cargo_bin("aget").unwrap();
    let empty_output = empty_cmd
        .env("AGET_HOME", &empty_home)
        .args(["--envelope", "json", "get", &empty_url, "--timeout", "60"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    empty_server.join().unwrap();

    let empty_json = success_data(&empty_output, "get");
    assert_eq!(empty_json["sessions"], serde_json::json!([]));
    assert_eq!(empty_json["sensitive"], false);
    assert!(empty_cookie
        .try_iter()
        .all(|cookie| !cookie.contains("sid=secret-cookie")));

    let session_home = temp.path().join("session-home");
    save_cookie_session(&session_home, "local", "127.0.0.1", "sid", "secret-cookie");
    let (session_url, session_server, session_cookie) = cookie_echo_server("/cookie-session");

    let mut session_cmd = Command::cargo_bin("aget").unwrap();
    let session_output = session_cmd
        .env("AGET_HOME", &session_home)
        .args([
            "--envelope",
            "json",
            "get",
            &session_url,
            "--session",
            "local",
            "--timeout",
            "60",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    session_server.join().unwrap();

    let session_json = success_data(&session_output, "get");
    assert_eq!(session_json["sessions"], serde_json::json!(["local"]));
    assert_eq!(session_json["sensitive"], true);
    assert!(session_cookie
        .try_iter()
        .any(|cookie| cookie.contains("sid=secret-cookie")));
}
