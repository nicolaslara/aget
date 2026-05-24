use assert_cmd::Command;
use predicates::prelude::*;

use super::support::local_server;

#[test]
fn top_level_url_runs_get_command() {
    let temp = tempfile::tempdir().unwrap();
    let (url, server) = local_server("<main># Example</main>");
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.env("AGET_HOME", temp.path().join("aget-home"))
        .arg(url)
        .assert()
        .success()
        .stdout(predicate::str::contains("# Example"));
    server.join().unwrap();
}

#[test]
fn top_level_url_alias_preserves_get_output_flags() {
    let temp = tempfile::tempdir().unwrap();
    let (url, server) = local_server("<main>Example</main>");
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .args(["--envelope", "json", &url, "--content-format", "html"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "get");
    assert_eq!(json["data"]["content_format"], "html");
    assert_eq!(json["data"]["content"], "<body><main>Example</main></body>");
    server.join().unwrap();
}

#[test]
fn envelope_global_flag_emits_structured_output() {
    let temp = tempfile::tempdir().unwrap();
    let (url, server) = local_server("<main># Example</main>");
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .args(["--envelope", "json", "get", &url])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "get");
    assert_eq!(json["data"]["content"], "# Example");
    server.join().unwrap();
}
