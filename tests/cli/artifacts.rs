use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

#[test]
fn artifacts_list_and_inspect_report_run_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let external_output = temp.path().join("caller-output.md");
    let run_id = create_run(&aget_home, Some(&external_output));

    let list = artifact_command(&aget_home)
        .args(["--envelope", "json", "artifacts", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list = success_data(&list, "artifacts.list");
    assert_eq!(list["summary"]["count"], 1);
    assert_eq!(list["runs"][0]["run_id"], run_id);
    assert_eq!(list["runs"][0]["content"]["ownership"], "external");
    assert_eq!(list["runs"][0]["content"]["exists"], true);
    assert_eq!(list["runs"][0]["source"]["url"], artifact_url());

    let inspect = artifact_command(&aget_home)
        .args(["--envelope", "json", "artifacts", "inspect", &run_id])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect = success_data(&inspect, "artifacts.inspect");
    assert_eq!(inspect["run_id"], run_id);
    assert_eq!(inspect["metadata"]["valid"], true);
    assert_eq!(inspect["metadata"]["value"]["url"], artifact_url());
    assert!(inspect["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file["kind"] == "content" && file["ownership"] == "external"));
}

#[test]
fn artifacts_delete_requires_yes_and_preserves_external_output() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let external_output = temp.path().join("caller-output.md");
    let run_id = create_run(&aget_home, Some(&external_output));
    let run_dir = aget_home.join("runs").join(&run_id);

    let output = artifact_command(&aget_home)
        .args(["--envelope", "json", "artifacts", "delete", &run_id])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "artifacts.delete");
    assert_eq!(error["error"]["code"], "usage_error");
    assert!(run_dir.exists());
    assert!(external_output.exists());

    let output = artifact_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "artifacts",
            "delete",
            &run_id,
            "--yes",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json = success_data(&output, "artifacts.delete");
    assert_eq!(json["deleted"], true);
    assert_eq!(
        json["preserved_external_paths"][0].as_str().unwrap(),
        external_output.to_string_lossy()
    );
    assert!(!run_dir.exists());
    assert_eq!(
        fs::read_to_string(&external_output).unwrap(),
        "# Artifact\n"
    );
}

#[test]
fn artifacts_prune_defaults_to_dry_run_and_deletes_with_yes() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let first = create_run(&aget_home, None);
    std::thread::sleep(std::time::Duration::from_secs(1));
    let second = create_run(&aget_home, None);

    let output = artifact_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "artifacts",
            "prune",
            "--keep-last",
            "1",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json = success_data(&output, "artifacts.prune");
    assert_eq!(json["dry_run"], true);
    assert_eq!(json["would_delete"], serde_json::json!([first]));
    assert!(aget_home.join("runs").join(&first).exists());
    assert!(aget_home.join("runs").join(&second).exists());

    let output = artifact_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "artifacts",
            "prune",
            "--keep-last",
            "1",
            "--yes",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json = success_data(&output, "artifacts.prune");
    assert_eq!(json["dry_run"], false);
    assert_eq!(json["deleted"], serde_json::json!([first]));
    assert!(!aget_home.join("runs").join(&first).exists());
    assert!(aget_home.join("runs").join(&second).exists());
}

#[test]
fn artifacts_prune_rejects_missing_selector_and_conflicting_confirmation() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let output = artifact_command(&aget_home)
        .args(["--envelope", "json", "artifacts", "prune"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "artifacts.prune");
    assert_eq!(error["error"]["code"], "usage_error");

    let output = artifact_command(&aget_home)
        .args([
            "--envelope",
            "json",
            "artifacts",
            "prune",
            "--keep-last",
            "1",
            "--dry-run",
            "--yes",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let error: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(error["command"], "artifacts.prune");
    assert_eq!(error["error"]["code"], "usage_error");
}

fn artifact_command(aget_home: &Path) -> Command {
    let mut command = Command::cargo_bin("aget").unwrap();
    command.env("AGET_HOME", aget_home);
    command
}

fn create_run(aget_home: &Path, external_output: Option<&Path>) -> String {
    let mut command = artifact_command(aget_home);
    command.args(["--envelope", "json", "get", &artifact_url()]);
    if let Some(output) = external_output {
        command.args(["--output", output.to_str().unwrap()]);
    }
    let output = command.assert().success().get_output().stdout.clone();
    let json = success_data(&output, "get");
    let metadata = PathBuf::from(json["artifacts"]["metadata"].as_str().unwrap());
    metadata
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap()
        .to_string()
}

fn artifact_url() -> String {
    "raw:<main><h1>Artifact</h1></main>".to_string()
}

fn success_data(output: &[u8], command: &str) -> serde_json::Value {
    let envelope: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], command);
    envelope["data"].clone()
}
