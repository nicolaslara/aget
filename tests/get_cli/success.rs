use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;

use crate::support::get_cli::{success_data, success_envelope};

#[test]
fn get_json_success_writes_run_artifacts_with_empty_state() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let local_url = "raw:<main><h1>Example</h1><p>Fetched locally.</p></main>";

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "get", local_url])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope = success_envelope(&output, "get");
    let json = &envelope["data"];
    assert_eq!(json["url"].as_str().unwrap(), local_url);
    assert_eq!(json["final_url"].as_str().unwrap(), local_url);
    assert_eq!(json["content_format"], "markdown");
    assert_eq!(json["extractor"], "aget-owned-extractor");
    assert_eq!(json["content"], "# Example\n\nFetched locally.");
    assert_eq!(json["sessions"], serde_json::json!([]));
    assert_eq!(json["sensitive"], false);
    assert_eq!(envelope["warnings"], serde_json::json!([]));
    assert_eq!(json["limits"]["max_chars"], serde_json::Value::Null);
    assert_eq!(json["limits"]["truncated"], false);

    let content_path = PathBuf::from(json["artifacts"]["content"].as_str().unwrap());
    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    assert_eq!(
        fs::read_to_string(&content_path).unwrap(),
        "# Example\n\nFetched locally.\n"
    );
    assert!(metadata_path.starts_with(aget_home.join("runs")));

    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["ok"], true);
    assert_eq!(metadata["extractor"], "aget-owned-extractor");
    assert_eq!(
        metadata["artifacts"]["content"],
        content_path.to_string_lossy().as_ref()
    );
    assert!(fs::read_dir(aget_home.join("tmp"))
        .unwrap()
        .next()
        .is_none());
}

#[test]
fn get_out_writes_markdown_to_requested_path_and_metadata_to_run_dir() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let out_path = temp.path().join("requested.md");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            "raw:<main><h1>Fake</h1></main>",
            "--output",
            out_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["extractor"], "aget-owned-extractor");
    assert_eq!(
        json["artifacts"]["content"].as_str().unwrap(),
        out_path.to_string_lossy().as_ref()
    );
    assert_eq!(fs::read_to_string(&out_path).unwrap(), "# Fake\n");
    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    assert!(metadata_path.starts_with(aget_home.join("runs")));
}

#[test]
fn get_inline_content_can_be_omitted_for_public_fetches() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            "raw:<main><h1>Public But Artifact Only</h1></main>",
            "--inline-content",
            "never",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["sensitive"], false);
    assert!(!json.as_object().unwrap().contains_key("content"));
    let content_path = PathBuf::from(json["artifacts"]["content"].as_str().unwrap());
    assert_eq!(
        fs::read_to_string(content_path).unwrap(),
        "# Public But Artifact Only\n"
    );
}

#[test]
fn get_writes_opt_in_trace_and_reports_static_screenshot_skip() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            "raw:<main><h1>Debug</h1><p>Trace only.</p></main>",
            "--capture-screenshot",
            "--capture-trace",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope = success_envelope(&output, "get");
    assert!(envelope["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning
            .as_str()
            .unwrap()
            .contains("no browser-rendered screenshot")));
    let json = &envelope["data"];
    assert!(json["artifacts"]["debug"].get("screenshot").is_none());
    let trace_path = PathBuf::from(
        json["artifacts"]["debug"]["trace"]["path"]
            .as_str()
            .unwrap(),
    );
    assert!(trace_path.starts_with(aget_home.join("runs")));
    let trace: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&trace_path).unwrap()).unwrap();
    assert_eq!(trace["schema_version"], "aget.debug_trace.v1");
    assert_eq!(trace["ok"], true);
    assert_eq!(trace["capture"]["screenshot_requested"], true);
    assert_eq!(trace["capture"]["screenshot_captured"], false);
    assert!(trace.get("content").is_none());
}

#[test]
fn get_raw_html_handles_edge_case_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let cases = [
        (
            "raw:Just plain text, no HTML tags",
            "Just plain text, no HTML tags",
        ),
        (
            "raw:<main><h1>Broken</h1><p>Unclosed paragraph",
            "# Broken\n\nUnclosed paragraph",
        ),
        (
            "raw:<main><p>Unicode 日本語 中文 한국어 العربية text</p></main>",
            "Unicode 日本語 中文 한국어 العربية text",
        ),
        (
            r##"raw:<main><a href="#section1">Jump</a><section id="section1">Fragment target</section></main>"##,
            "[Jump](#section1) Fragment target",
        ),
    ];

    for (input, expected) in cases {
        let output = Command::cargo_bin("aget")
            .unwrap()
            .env("AGET_HOME", &aget_home)
            .args(["--envelope", "json", "get", input])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let json = success_data(&output, "get");
        assert_eq!(json["content"], expected, "raw input failed: {input}");
    }
}
