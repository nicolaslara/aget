use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_includes_global_flags() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.arg("--help").assert().success().stdout(
        predicate::str::contains("--json")
            .and(predicate::str::contains("--timeout"))
            .and(predicate::str::contains("get")),
    );
}

#[test]
fn get_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: aget get").and(predicate::str::contains("<URL>")));
}

#[test]
fn top_level_url_runs_get_command() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.arg("https://example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("aget get https://example.com"));
}
