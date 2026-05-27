use std::path::PathBuf;

use clap::Parser;

use super::super::*;

#[test]
fn parses_setup_skills_defaults() {
    let cli = Cli::try_parse_from(["aget", "setup-skills"]).unwrap();

    assert_eq!(
        cli.command,
        Command::SetupSkills(SetupSkillsCommand {
            all: false,
            codex: false,
            claude: false,
            gemini: false,
            windsurf: false,
            opencode: false,
            cursor: false,
            copilot: false,
            dry_run: false,
            force: false,
            copy: false,
            symlink: false,
            project_dir: None,
            codex_home: None,
            claude_home: None,
            gemini_home: None,
            windsurf_home: None,
        })
    );
}

#[test]
fn parses_setup_skills_all_project_options() {
    let cli = Cli::try_parse_from([
        "aget",
        "setup-skills",
        "--all",
        "--dry-run",
        "--force",
        "--copy",
        "--project-dir",
        "/tmp/project",
        "--codex-home",
        "/tmp/codex",
        "--claude-home",
        "/tmp/claude",
        "--gemini-home",
        "/tmp/gemini",
        "--windsurf-home",
        "/tmp/windsurf",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::SetupSkills(SetupSkillsCommand {
            all: true,
            codex: false,
            claude: false,
            gemini: false,
            windsurf: false,
            opencode: false,
            cursor: false,
            copilot: false,
            dry_run: true,
            force: true,
            copy: true,
            symlink: false,
            project_dir: Some(PathBuf::from("/tmp/project")),
            codex_home: Some(PathBuf::from("/tmp/codex")),
            claude_home: Some(PathBuf::from("/tmp/claude")),
            gemini_home: Some(PathBuf::from("/tmp/gemini")),
            windsurf_home: Some(PathBuf::from("/tmp/windsurf")),
        })
    );
}
