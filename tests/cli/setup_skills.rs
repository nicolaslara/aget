use std::fs;

use assert_cmd::Command;

#[test]
fn setup_skills_dry_run_reports_all_targets_without_writing() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();

    let output = Command::cargo_bin("aget")
        .unwrap()
        .args([
            "--envelope",
            "json",
            "setup-skills",
            "--all",
            "--dry-run",
            "--project-dir",
        ])
        .arg(&project)
        .args(custom_home_args(temp.path()))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["command"], "setup-skills");
    assert_eq!(envelope["data"]["dry_run"], true);
    assert_eq!(envelope["data"]["summary"]["total"], 8);
    assert_eq!(envelope["data"]["summary"]["planned"], 8);
    assert!(envelope["data"]["integrations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == "opencode"));
    assert!(!temp.path().join("codex/skills/aget/SKILL.md").exists());
    assert!(!project.join(".opencode/tools/aget.ts").exists());
}

#[test]
fn setup_skills_installs_global_skills_and_project_adapters() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    fs::create_dir_all(&project).unwrap();

    Command::cargo_bin("aget")
        .unwrap()
        .args(["setup-skills", "--all", "--force", "--project-dir"])
        .arg(&project)
        .args(custom_home_args(temp.path()))
        .assert()
        .success();

    for path in [
        temp.path().join("codex/skills/aget/SKILL.md"),
        temp.path().join("claude/skills/aget/SKILL.md"),
        temp.path().join("gemini/skills/aget/SKILL.md"),
        temp.path().join("windsurf/skills/aget/SKILL.md"),
        project.join(".opencode/tools/aget.ts"),
        project.join(".opencode/lib/aget_args.ts"),
        project.join(".cursor/rules/aget.mdc"),
        project.join(".github/instructions/aget.instructions.md"),
    ] {
        let content = fs::read_to_string(&path).unwrap();
        assert!(
            !content.trim().is_empty(),
            "empty integration file {path:?}"
        );
    }
}

#[test]
fn setup_skills_rejects_existing_target_without_force() {
    let temp = tempfile::tempdir().unwrap();

    Command::cargo_bin("aget")
        .unwrap()
        .args(["setup-skills", "--codex", "--codex-home"])
        .arg(temp.path().join("codex"))
        .assert()
        .success();

    let output = Command::cargo_bin("aget")
        .unwrap()
        .args([
            "--envelope",
            "json",
            "setup-skills",
            "--codex",
            "--codex-home",
        ])
        .arg(temp.path().join("codex"))
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["command"], "setup-skills");
    assert_eq!(envelope["error"]["code"], "usage_error");
    assert!(envelope["error"]["message"]
        .as_str()
        .unwrap()
        .contains("--force"));
}

#[test]
fn setup_skills_requires_project_dir_for_project_adapters() {
    let output = Command::cargo_bin("aget")
        .unwrap()
        .args(["--envelope", "json", "setup-skills", "--cursor"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["command"], "setup-skills");
    assert_eq!(envelope["error"]["code"], "usage_error");
    assert!(envelope["error"]["message"]
        .as_str()
        .unwrap()
        .contains("--project-dir"));
}

#[test]
fn setup_skills_rejects_symlink_mode_from_installed_binary() {
    let output = Command::cargo_bin("aget")
        .unwrap()
        .args(["--envelope", "json", "setup-skills", "--codex", "--symlink"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let envelope: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(envelope["command"], "setup-skills");
    assert_eq!(envelope["error"]["code"], "usage_error");
    assert!(envelope["error"]["message"]
        .as_str()
        .unwrap()
        .contains("embedded templates"));
}

fn custom_home_args(root: &std::path::Path) -> Vec<std::ffi::OsString> {
    vec![
        "--codex-home".into(),
        root.join("codex").into_os_string(),
        "--claude-home".into(),
        root.join("claude").into_os_string(),
        "--gemini-home".into(),
        root.join("gemini").into_os_string(),
        "--windsurf-home".into(),
        root.join("windsurf").into_os_string(),
    ]
}
