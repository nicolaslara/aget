use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_includes_global_flags() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.arg("--help").assert().success().stdout(
        predicate::str::contains("--json")
            .and(predicate::str::contains("--envelope"))
            .and(predicate::str::contains("--timeout"))
            .and(predicate::str::contains("get")),
    );
}

#[test]
fn get_help_is_available() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["get", "--help"]).assert().success().stdout(
        predicate::str::contains("Usage: aget get")
            .and(predicate::str::contains("<URL>"))
            .and(predicate::str::contains("--format"))
            .and(predicate::str::contains("--selector"))
            .and(predicate::str::contains("--exclude-selector"))
            .and(predicate::str::contains("--wait-for"))
            .and(predicate::str::contains("--max-chars"))
            .and(predicate::str::contains("--extractor-option")),
    );
}

#[test]
fn get_rejects_invalid_format() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.args(["get", "https://example.com", "--format", "pdf"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn get_json_parse_error_uses_structured_envelope() {
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .args(["--json", "get", "https://example.com", "--format", "pdf"])
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
        "--extractor-option",
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
        "--extractor-option",
        "wait_until=networkidle",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("must be namespaced"));
}

#[test]
fn top_level_url_runs_get_command() {
    let temp = tempfile::tempdir().unwrap();
    let fake_backend = temp.path().join("fake_backend.py");
    std::fs::write(
        &fake_backend,
        r#"#!/usr/bin/env python3
import argparse, json, pathlib
parser = argparse.ArgumentParser()
parser.add_argument('--url', required=True)
parser.add_argument('--state', required=True)
parser.add_argument('--output', required=True)
parser.add_argument('--metadata', required=True)
args, _unknown = parser.parse_known_args()
content = '# Example\n'
pathlib.Path(args.output).write_text(content, encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': args.url, 'content': content, 'warnings': []}))
"#,
    )
    .unwrap();
    let mut cmd = Command::cargo_bin("aget").unwrap();

    cmd.env("AGET_HOME", temp.path().join("aget-home"))
        .env(
            "AGET_CRAWL4AI_COMMAND",
            format!("python3 {}", shell_quote(&fake_backend.to_string_lossy())),
        )
        .arg("https://example.com")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Example"));
}

#[test]
fn top_level_url_alias_preserves_get_output_flags() {
    let temp = tempfile::tempdir().unwrap();
    let fake_backend = temp.path().join("fake_backend.py");
    std::fs::write(
        &fake_backend,
        r#"#!/usr/bin/env python3
import argparse, json, pathlib
parser = argparse.ArgumentParser()
parser.add_argument('--url', required=True)
parser.add_argument('--state', required=True)
parser.add_argument('--output', required=True)
parser.add_argument('--metadata', required=True)
parser.add_argument('--format', required=True)
args, _unknown = parser.parse_known_args()
content = '<main>Example</main>'
pathlib.Path(args.output).write_text(content, encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': args.url, 'content': content, 'warnings': []}))
"#,
    )
    .unwrap();
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .env(
            "AGET_CRAWL4AI_COMMAND",
            format!("python3 {}", shell_quote(&fake_backend.to_string_lossy())),
        )
        .args(["--json", "https://example.com", "--format", "html"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "get");
    assert_eq!(json["data"]["format"], "html");
    assert_eq!(json["data"]["content"], "<main>Example</main>");
}

#[test]
fn envelope_global_flag_emits_structured_output() {
    let temp = tempfile::tempdir().unwrap();
    let fake_backend = temp.path().join("fake_backend.py");
    std::fs::write(
        &fake_backend,
        r#"#!/usr/bin/env python3
import argparse, json, pathlib
parser = argparse.ArgumentParser()
parser.add_argument('--url', required=True)
parser.add_argument('--state', required=True)
parser.add_argument('--output', required=True)
parser.add_argument('--metadata', required=True)
args, _unknown = parser.parse_known_args()
content = '# Example\n'
pathlib.Path(args.output).write_text(content, encoding='utf-8')
print(json.dumps({'ok': True, 'final_url': args.url, 'content': content, 'warnings': []}))
"#,
    )
    .unwrap();
    let mut cmd = Command::cargo_bin("aget").unwrap();

    let output = cmd
        .env("AGET_HOME", temp.path().join("aget-home"))
        .env(
            "AGET_CRAWL4AI_COMMAND",
            format!("python3 {}", shell_quote(&fake_backend.to_string_lossy())),
        )
        .args(["--envelope", "get", "https://example.com"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], "get");
    assert_eq!(json["data"]["content"], "# Example\n");
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
