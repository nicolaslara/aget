use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use aget::{ErrorCode, ErrorResponse, SetupSkillsCommand, TimingMs};
use serde::Serialize;

use crate::main_envelope::print_success_envelope;

const SKILL_MD: &str = include_str!("../skills/aget/SKILL.md");
const OPENCODE_TOOL_TS: &str = include_str!("../.opencode/tools/aget.ts");
const OPENCODE_ARGS_TS: &str = include_str!("../.opencode/lib/aget_args.ts");
const CURSOR_RULE_MDC: &str = include_str!("../integrations/cursor/aget.mdc");
const COPILOT_INSTRUCTIONS_MD: &str = include_str!("../integrations/copilot/aget.instructions.md");

pub(crate) fn run_setup_skills(
    command: SetupSkillsCommand,
    json: bool,
    quiet: bool,
    started: Instant,
) -> Result<(), ErrorResponse> {
    if command.symlink {
        return Err(usage_error(
            "setup-skills --symlink is only supported by checkout scripts; installed binaries copy embedded templates",
        ));
    }

    let plan = build_plan(&command)?;
    let mut results = Vec::new();
    for item in plan {
        results.push(apply_item(item, command.force, command.dry_run)?);
    }

    let data = SetupSkillsData {
        dry_run: command.dry_run,
        summary: SetupSkillsSummary::from_results(&results),
        integrations: results,
    };

    if json {
        print_success_envelope(
            "setup-skills",
            serde_json::to_value(data).map_err(io_error)?,
            Vec::new(),
            elapsed_timing(started),
        )?;
    } else if !quiet {
        for item in &data.integrations {
            println!(
                "{} {} {}",
                item.status.human_label(),
                item.integration,
                item.path.display()
            );
        }
        if data.dry_run {
            println!("Dry run: no files were changed.");
        }
        println!("Restart or reload each agent harness to pick up installed integrations.");
    }

    Ok(())
}

fn build_plan(command: &SetupSkillsCommand) -> Result<Vec<SetupPlanItem>, ErrorResponse> {
    let any_target = command.codex
        || command.claude
        || command.gemini
        || command.windsurf
        || command.opencode
        || command.cursor
        || command.copilot;
    let install_defaults = !any_target || command.all;
    let install_project_defaults = command.all && command.project_dir.is_some();

    let mut items = Vec::new();
    if install_defaults || command.codex {
        items.push(skill_item(
            "codex",
            "Codex skill",
            codex_home(command)?.join("skills/aget"),
        ));
    }
    if install_defaults || command.claude {
        items.push(skill_item(
            "claude",
            "Claude Code skill",
            home_path(command.claude_home.as_ref(), ".claude")?.join("skills/aget"),
        ));
    }
    if install_defaults || command.gemini {
        items.push(skill_item(
            "gemini",
            "Gemini CLI skill",
            home_path(command.gemini_home.as_ref(), ".gemini")?.join("skills/aget"),
        ));
    }
    if install_defaults || command.windsurf {
        items.push(skill_item(
            "windsurf",
            "Windsurf skill",
            home_path(command.windsurf_home.as_ref(), ".codeium/windsurf")?.join("skills/aget"),
        ));
    }

    if install_project_defaults || command.opencode {
        let project = project_dir(command)?;
        items.push(file_item(
            "opencode",
            "OpenCode aget tool",
            project.join(".opencode/tools/aget.ts"),
            OPENCODE_TOOL_TS,
        ));
        items.push(file_item(
            "opencode",
            "OpenCode aget argument helper",
            project.join(".opencode/lib/aget_args.ts"),
            OPENCODE_ARGS_TS,
        ));
    }
    if install_project_defaults || command.cursor {
        items.push(file_item(
            "cursor",
            "Cursor aget rule",
            project_dir(command)?.join(".cursor/rules/aget.mdc"),
            CURSOR_RULE_MDC,
        ));
    }
    if install_project_defaults || command.copilot {
        items.push(file_item(
            "copilot",
            "Copilot aget instructions",
            project_dir(command)?.join(".github/instructions/aget.instructions.md"),
            COPILOT_INSTRUCTIONS_MD,
        ));
    }

    if items.is_empty() {
        return Err(usage_error("no setup-skills targets selected"));
    }
    Ok(items)
}

fn skill_item(id: &'static str, integration: &'static str, root: PathBuf) -> SetupPlanItem {
    SetupPlanItem {
        id,
        integration,
        path: root.join("SKILL.md"),
        replace_path: root,
        content: SKILL_MD,
    }
}

fn file_item(
    id: &'static str,
    integration: &'static str,
    path: PathBuf,
    content: &'static str,
) -> SetupPlanItem {
    SetupPlanItem {
        id,
        integration,
        replace_path: path.clone(),
        path,
        content,
    }
}

fn codex_home(command: &SetupSkillsCommand) -> Result<PathBuf, ErrorResponse> {
    if let Some(path) = &command.codex_home {
        return Ok(path.clone());
    }
    if let Some(path) = env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(path));
    }
    home_path(None, ".codex")
}

fn home_path(explicit: Option<&PathBuf>, suffix: &str) -> Result<PathBuf, ErrorResponse> {
    if let Some(path) = explicit {
        return Ok(path.clone());
    }
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| usage_error("HOME is not set; pass an explicit home directory option"))?;
    Ok(home.join(suffix))
}

fn project_dir(command: &SetupSkillsCommand) -> Result<PathBuf, ErrorResponse> {
    command
        .project_dir
        .clone()
        .ok_or_else(|| usage_error("--opencode, --cursor, and --copilot require --project-dir DIR"))
}

fn apply_item(
    item: SetupPlanItem,
    force: bool,
    dry_run: bool,
) -> Result<SetupInstallItem, ErrorResponse> {
    let exists = item.replace_path.exists() || item.replace_path.symlink_metadata().is_ok();
    let status = if dry_run {
        if exists {
            SetupInstallStatus::Exists
        } else {
            SetupInstallStatus::Planned
        }
    } else if exists && !force {
        return Err(usage_error(format!(
            "{} already exists; pass --force to replace it",
            item.path.display()
        )));
    } else {
        write_embedded_file(
            &item.path,
            item.content,
            exists.then_some(item.replace_path.as_path()),
        )?;
        SetupInstallStatus::Installed
    };

    Ok(SetupInstallItem {
        id: item.id,
        integration: item.integration,
        path: item.path,
        status,
    })
}

fn write_embedded_file(
    path: &Path,
    content: &str,
    replace_existing: Option<&Path>,
) -> Result<(), ErrorResponse> {
    if let Some(replace_existing) = replace_existing {
        remove_existing(replace_existing)?;
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    fs::write(path, content).map_err(io_error)
}

fn remove_existing(path: &Path) -> Result<(), ErrorResponse> {
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path).map_err(io_error)
    } else {
        fs::remove_file(path).map_err(io_error)
    }
}

fn elapsed_timing(started: Instant) -> TimingMs {
    TimingMs {
        total: started.elapsed().as_millis(),
    }
}

fn usage_error(message: impl Into<String>) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::UsageError, message)
}

fn io_error(error: impl std::fmt::Display) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::IoError, error.to_string())
}

struct SetupPlanItem {
    id: &'static str,
    integration: &'static str,
    path: PathBuf,
    replace_path: PathBuf,
    content: &'static str,
}

#[derive(Debug, Serialize)]
struct SetupSkillsData {
    dry_run: bool,
    summary: SetupSkillsSummary,
    integrations: Vec<SetupInstallItem>,
}

#[derive(Debug, Serialize)]
struct SetupSkillsSummary {
    total: usize,
    installed: usize,
    planned: usize,
    existing: usize,
}

impl SetupSkillsSummary {
    fn from_results(results: &[SetupInstallItem]) -> Self {
        Self {
            total: results.len(),
            installed: results
                .iter()
                .filter(|item| item.status == SetupInstallStatus::Installed)
                .count(),
            planned: results
                .iter()
                .filter(|item| item.status == SetupInstallStatus::Planned)
                .count(),
            existing: results
                .iter()
                .filter(|item| item.status == SetupInstallStatus::Exists)
                .count(),
        }
    }
}

#[derive(Debug, Serialize)]
struct SetupInstallItem {
    id: &'static str,
    integration: &'static str,
    path: PathBuf,
    status: SetupInstallStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SetupInstallStatus {
    Planned,
    Installed,
    Exists,
}

impl SetupInstallStatus {
    fn human_label(self) -> &'static str {
        match self {
            Self::Planned => "would install",
            Self::Installed => "installed",
            Self::Exists => "exists",
        }
    }
}
