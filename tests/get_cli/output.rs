use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;

use crate::support::get_cli::{success_data, success_envelope};

#[test]
fn get_applies_supported_output_options_and_records_limits() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            "raw:<main class=\"article\">aé💡bc</main><nav>noise</nav>",
            "--content-format",
            "text",
            "--selector",
            "main.article",
            "--exclude-selector",
            "nav,.ad",
            "--max-chars",
            "3",
            "--backend-option",
            "aget.ignore_links=true",
            "--backend-option",
            "aget.body_width=40",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["content_format"], "text");
    assert_eq!(json["content"], "aé💡");
    assert_eq!(json["limits"]["max_chars"], 3);
    assert_eq!(json["limits"]["truncated"], true);
    assert_eq!(json["limits"]["truncated_by"], "max_chars");
    assert_eq!(json["limits"]["content_chars_before_truncation"], 5);
    assert_eq!(json["limits"]["content_chars_after_truncation"], 3);
    assert_eq!(json["output_options"]["content_format"], "text");
    assert_eq!(json["output_options"]["selector"], "main.article");
    assert_eq!(json["output_options"]["exclude_selector"], "nav,.ad");
    assert_eq!(
        json["output_options"]["backend_options"],
        serde_json::json!({"aget.body_width": "40", "aget.ignore_links": "true"})
    );

    let content_path = PathBuf::from(json["artifacts"]["content"].as_str().unwrap());
    assert_eq!(fs::read_to_string(&content_path).unwrap(), "aé💡\n");
    let metadata_path = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["content"], serde_json::Value::Null);
    assert_eq!(metadata["output_options"], json["output_options"]);
    assert_eq!(metadata["limits"], json["limits"]);
}

#[test]
fn get_max_chars_sanitizes_artifacts_to_truncated_content() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            "raw:<main>SECRET-UNTRUNCATED-CONTENT</main>",
            "--max-chars",
            "6",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["content"], "SECRET");
    assert_eq!(json["limits"]["truncated"], true);

    let content_path = PathBuf::from(json["artifacts"]["content"].as_str().unwrap());
    let artifact = fs::read_to_string(content_path).unwrap();
    assert_eq!(artifact, "SECRET\n");
    assert!(!artifact.contains("SECRET-UNTRUNCATED-CONTENT"));
}

#[test]
fn get_json_format_truncates_only_content_not_response_envelope() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            "raw:<main>{\"title\":\"Example\",\"body\":\"abcdef\"}</main>",
            "--content-format",
            "json",
            "--max-chars",
            "12",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope = success_envelope(&output, "get");
    let json = &envelope["data"];
    assert_eq!(json["content_format"], "json");
    assert_eq!(json["content"], "{\"content\":\"");
    assert_eq!(json["limits"]["truncated"], true);
    assert!(!json["artifacts"]["metadata"].as_str().unwrap().is_empty());
    assert!(envelope["timing_ms"]["total"].as_u64().is_some());
}
