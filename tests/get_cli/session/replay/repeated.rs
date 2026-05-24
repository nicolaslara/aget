use assert_cmd::Command;

use crate::support::get_cli::{cookie_echo_server, save_cookie_session, success_data};

#[test]
fn get_repeated_sessions_compose_request_state_for_command_and_alias() {
    let temp = tempfile::tempdir().unwrap();
    let command_home = temp.path().join("command-home");
    let (command_url, command_server, command_cookies) = cookie_echo_server("/multi");
    save_cookie_session(
        &command_home,
        "provider",
        "127.0.0.1",
        "oauth",
        "provider-secret",
    );
    save_cookie_session(&command_home, "app", "127.0.0.1", "appsid", "app-secret");

    let mut command = Command::cargo_bin("aget").unwrap();
    let command_output = command
        .env("AGET_HOME", &command_home)
        .args([
            "--envelope",
            "json",
            "get",
            &command_url,
            "--session",
            "provider",
            "--session",
            "app",
            "--inline-content",
            "always",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    command_server.join().unwrap();
    let command_headers = command_cookies.try_iter().collect::<Vec<_>>();
    assert!(command_headers
        .iter()
        .any(|cookie| cookie.contains("oauth=provider-secret")));
    assert!(command_headers
        .iter()
        .any(|cookie| cookie.contains("appsid=app-secret")));
    let command_json = success_data(&command_output, "get");
    assert_eq!(
        command_json["sessions"],
        serde_json::json!(["provider", "app"])
    );
    assert_eq!(command_json["sensitive"], true);

    let alias_home = temp.path().join("alias-home");
    let (alias_url, alias_server, alias_cookies) = cookie_echo_server("/multi-alias");
    save_cookie_session(
        &alias_home,
        "provider",
        "127.0.0.1",
        "oauth",
        "provider-secret",
    );
    save_cookie_session(&alias_home, "app", "127.0.0.1", "appsid", "app-secret");
    let mut alias = Command::cargo_bin("aget").unwrap();
    let alias_output = alias
        .env("AGET_HOME", &alias_home)
        .args([
            "--envelope",
            "json",
            &alias_url,
            "--session",
            "provider",
            "--session",
            "app",
            "--inline-content",
            "always",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    alias_server.join().unwrap();
    let alias_headers = alias_cookies.try_iter().collect::<Vec<_>>();
    assert!(alias_headers
        .iter()
        .any(|cookie| cookie.contains("oauth=provider-secret")));
    assert!(alias_headers
        .iter()
        .any(|cookie| cookie.contains("appsid=app-secret")));
    let alias_json = success_data(&alias_output, "get");
    assert_eq!(
        alias_json["sessions"],
        serde_json::json!(["provider", "app"])
    );
}
