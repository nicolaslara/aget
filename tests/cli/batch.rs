use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

#[test]
fn batch_fetches_urls_and_writes_manifest() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let output_dir = temp.path().join("batch-output");

    let output = batch_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "batch",
            "raw:<main><h1>One</h1></main>",
            "raw:<main><h1>Two</h1></main>",
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--concurrency",
            "2",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_summary(&data, 2, 2, 0, 0);
    assert_eq!(data["items"][0]["status"], "ok");
    assert_eq!(data["items"][1]["status"], "ok");

    let manifest = path_from(&data["artifacts"]["manifest"]);
    let markdown = path_from(&data["artifacts"]["markdown"]);
    assert_eq!(manifest, output_dir.join("manifest.json"));
    assert_eq!(markdown, output_dir.join("manifest.md"));
    assert!(manifest.exists());
    assert!(markdown.exists());
    assert!(path_from(&data["items"][0]["artifacts"]["content"]).exists());
    assert!(path_from(&data["items"][0]["artifacts"]["metadata"]).exists());
    assert!(
        fs::read_to_string(path_from(&data["items"][1]["artifacts"]["content"]))
            .unwrap()
            .contains("# Two")
    );
}

#[test]
fn batch_reads_file_input_and_marks_duplicates_skipped() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let output_dir = temp.path().join("batch-output");
    let input_file = temp.path().join("urls.txt");
    fs::write(
        &input_file,
        "\n# comment\nraw:<main><h1>One</h1></main>\nraw:<main><h1>One</h1></main>\n",
    )
    .unwrap();

    let output = batch_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "batch",
            "--file",
            input_file.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_summary(&data, 2, 1, 0, 1);
    assert_eq!(data["items"][1]["status"], "skipped");
    assert_eq!(data["items"][1]["reason"], "duplicate");
}

#[test]
fn batch_reads_stdin_input() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let output_dir = temp.path().join("batch-output");

    let output = batch_command(&aget_home)
        .write_stdin("raw:<main><h1>From stdin</h1></main>\n")
        .args([
            "--envelope",
            "json",
            "batch",
            "--stdin",
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let data = success_data(&output);
    assert_summary(&data, 1, 1, 0, 0);
    assert!(
        fs::read_to_string(path_from(&data["items"][0]["artifacts"]["content"]))
            .unwrap()
            .contains("# From stdin")
    );
}

#[test]
fn batch_reports_partial_failure_deterministically() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let output_dir = temp.path().join("batch-output");
    let missing = format!("file://{}", temp.path().join("missing.html").display());

    let output = batch_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "batch",
            "raw:<main><h1>Ok</h1></main>",
            &missing,
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--concurrency",
            "2",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["ok"], false);
    assert_eq!(envelope["command"], "batch");
    let data = envelope["data"].clone();
    assert_summary(&data, 2, 1, 1, 0);
    assert_eq!(data["items"][0]["status"], "ok");
    assert_eq!(data["items"][1]["status"], "failed");
    assert!(matches!(
        data["items"][1]["error"]["code"].as_str(),
        Some("io_error" | "extraction_failed")
    ));
    assert!(path_from(&data["artifacts"]["manifest"]).exists());
}

#[test]
fn batch_rejects_invalid_input_shape_with_structured_error() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let output = batch_command(&aget_home)
        .args(["--envelope", "json", "batch", "--concurrency", "0"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "batch");
    assert_eq!(error["error"]["code"], "usage_error");
}

fn batch_command(aget_home: &Path) -> Command {
    let mut command = Command::cargo_bin("aget").unwrap();
    command.env("AGET_HOME", aget_home);
    command
}

fn success_data(output: &[u8]) -> serde_json::Value {
    let envelope: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "batch");
    envelope["data"].clone()
}

fn assert_summary(
    data: &serde_json::Value,
    requested: usize,
    succeeded: usize,
    failed: usize,
    skipped: usize,
) {
    assert_eq!(data["summary"]["requested"], requested);
    assert_eq!(data["summary"]["succeeded"], succeeded);
    assert_eq!(data["summary"]["failed"], failed);
    assert_eq!(data["summary"]["skipped"], skipped);
}

fn path_from(value: &serde_json::Value) -> PathBuf {
    PathBuf::from(value.as_str().unwrap())
}
