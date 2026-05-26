use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_includes_global_flags() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.arg("--help").assert().success().stdout(
        predicate::str::contains("--envelope")
            .and(predicate::str::contains("--timeout"))
            .and(predicate::str::contains("get"))
            .and(predicate::str::contains("interact"))
            .and(predicate::str::contains("search-page")),
    );
}

#[test]
fn get_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["get", "--help"]).assert().success().stdout(
        predicate::str::contains("Usage: aget get")
            .and(predicate::str::contains("<URL>"))
            .and(predicate::str::contains("--content-format"))
            .and(predicate::str::contains("--inline-content"))
            .and(predicate::str::contains("--selector"))
            .and(predicate::str::contains("--exclude-selector"))
            .and(predicate::str::contains("--wait-for-selector"))
            .and(predicate::str::contains("--max-chars"))
            .and(predicate::str::contains("--capture-screenshot"))
            .and(predicate::str::contains("--capture-trace"))
            .and(predicate::str::contains("--backend-option")),
    );
}

#[test]
fn current_tab_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["current-tab", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Usage: aget current-tab")
                .and(predicate::str::contains("--cdp-port"))
                .and(predicate::str::contains("--allow-private-content"))
                .and(predicate::str::contains("--content-format"))
                .and(predicate::str::contains("--inline-content"))
                .and(predicate::str::contains("--capture-screenshot"))
                .and(predicate::str::contains("--capture-trace")),
        );
}

#[test]
fn doctor_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["doctor", "--help"]).assert().success().stdout(
        predicate::str::contains("Usage: aget doctor")
            .and(predicate::str::contains("--quick"))
            .and(predicate::str::contains("--check")),
    );
}

#[test]
fn artifacts_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["artifacts", "--help"]).assert().success().stdout(
        predicate::str::contains("Usage: aget artifacts")
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("inspect"))
            .and(predicate::str::contains("delete"))
            .and(predicate::str::contains("prune")),
    );
}

#[test]
fn batch_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["batch", "--help"]).assert().success().stdout(
        predicate::str::contains("Usage: aget batch")
            .and(predicate::str::contains("--file"))
            .and(predicate::str::contains("--stdin"))
            .and(predicate::str::contains("--concurrency"))
            .and(predicate::str::contains("--output-dir"))
            .and(predicate::str::contains("--fail-fast")),
    );
}

#[test]
fn map_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["map", "--help"]).assert().success().stdout(
        predicate::str::contains("Usage: aget map")
            .and(predicate::str::contains("--artifact"))
            .and(predicate::str::contains("--same-origin"))
            .and(predicate::str::contains("--any-origin"))
            .and(predicate::str::contains("--same-path"))
            .and(predicate::str::contains("--any-path"))
            .and(predicate::str::contains("--max-links"))
            .and(predicate::str::contains("--content-type")),
    );
}

#[test]
fn crawl_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["crawl", "--help"]).assert().success().stdout(
        predicate::str::contains("Usage: aget crawl")
            .and(predicate::str::contains("--limit"))
            .and(predicate::str::contains("--max-depth"))
            .and(predicate::str::contains("--concurrency"))
            .and(predicate::str::contains("--same-origin"))
            .and(predicate::str::contains("--any-origin"))
            .and(predicate::str::contains("--allow-domain"))
            .and(predicate::str::contains("--output-dir")),
    );
}

#[test]
fn search_page_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["search-page", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Usage: aget search-page")
                .and(predicate::str::contains("--artifact"))
                .and(predicate::str::contains("--query"))
                .and(predicate::str::contains("--max-results"))
                .and(predicate::str::contains("--allow-private-content")),
        );
}

#[test]
fn extract_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["extract", "--help"]).assert().success().stdout(
        predicate::str::contains("Usage: aget extract")
            .and(predicate::str::contains("--artifact"))
            .and(predicate::str::contains("--manifest"))
            .and(predicate::str::contains("--schema"))
            .and(predicate::str::contains("--field"))
            .and(predicate::str::contains("--allow-private-content")),
    );
}

#[test]
fn interact_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["interact", "--help"]).assert().success().stdout(
        predicate::str::contains("Usage: aget interact")
            .and(predicate::str::contains("--actions"))
            .and(predicate::str::contains("--allow-actions"))
            .and(predicate::str::contains("--allow-private-content"))
            .and(predicate::str::contains("--allow-sensitive-input"))
            .and(predicate::str::contains("--allow-submit"))
            .and(predicate::str::contains("--capture-screenshot"))
            .and(predicate::str::contains("--dry-run")),
    );
}

#[test]
fn get_rejects_invalid_format() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["get", "https://example.com", "--content-format", "pdf"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn get_json_parse_error_uses_structured_envelope() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com",
            "--content-format",
            "pdf",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["command"], "get");
    assert_eq!(json["error"]["code"], "usage_error");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("invalid value"));
}

#[test]
fn get_rejects_malformed_extractor_option() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args([
        "get",
        "https://example.com",
        "--backend-option",
        "missing-equals",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("key=value"));
}

#[test]
fn get_rejects_unnamespaced_extractor_option() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args([
        "get",
        "https://example.com",
        "--backend-option",
        "wait_until=networkidle",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("must be namespaced"));
}
