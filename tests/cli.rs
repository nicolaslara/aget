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
args = parser.parse_args()
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

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
