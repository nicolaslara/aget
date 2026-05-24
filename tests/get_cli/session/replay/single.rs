use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;

use crate::support::get_cli::{cookie_echo_server, save_cookie_session, success_data};

#[test]
fn get_session_uses_named_session_state_and_marks_sensitive() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let (url, server, cookies) = cookie_echo_server("/session");
    save_cookie_session(&aget_home, "local", "127.0.0.1", "sid", "secret-cookie");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "get", &url, "--session", "local"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    server.join().unwrap();

    let headers = cookies.try_iter().collect::<Vec<_>>();
    assert!(headers
        .iter()
        .any(|cookie| cookie.contains("sid=secret-cookie")));

    let json = success_data(&output, "get");
    assert_eq!(json["sessions"], serde_json::json!(["local"]));
    assert_eq!(json["sensitive"], true);
    assert!(!json.as_object().unwrap().contains_key("content"));

    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["sessions"], serde_json::json!(["local"]));
    assert_eq!(metadata["sensitive"], true);
}
