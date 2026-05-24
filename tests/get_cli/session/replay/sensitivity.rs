use assert_cmd::Command;

use crate::support::get_cli::{
    cookie_echo_server, save_cookie_session_with_sensitivity, success_data,
};

#[test]
fn get_session_marks_output_sensitive_even_if_session_metadata_is_false() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let (url, server, cookies) = cookie_echo_server("/session");
    save_cookie_session_with_sensitivity(
        &aget_home,
        "local",
        "127.0.0.1",
        "sid",
        "secret-cookie",
        false,
    );

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
    assert!(cookies.recv().unwrap().contains("sid=secret-cookie"));

    let json = success_data(&output, "get");
    assert_eq!(json["sessions"], serde_json::json!(["local"]));
    assert_eq!(json["sensitive"], true);
}
