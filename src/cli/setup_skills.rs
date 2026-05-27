use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct SetupSkillsCommand {
    /// Install every global integration, plus project adapters when --project-dir is present.
    #[arg(long)]
    pub all: bool,

    /// Install the Codex global skill.
    #[arg(long)]
    pub codex: bool,

    /// Install the Claude Code global skill.
    #[arg(long)]
    pub claude: bool,

    /// Install the Gemini CLI global skill.
    #[arg(long)]
    pub gemini: bool,

    /// Install the Windsurf global skill.
    #[arg(long)]
    pub windsurf: bool,

    /// Install OpenCode project-local tools. Requires --project-dir.
    #[arg(long)]
    pub opencode: bool,

    /// Install a Cursor project rule. Requires --project-dir.
    #[arg(long)]
    pub cursor: bool,

    /// Install GitHub Copilot project instructions. Requires --project-dir.
    #[arg(long)]
    pub copilot: bool,

    /// Print planned writes without changing files.
    #[arg(long)]
    pub dry_run: bool,

    /// Replace existing aget integrations.
    #[arg(long)]
    pub force: bool,

    /// Copy embedded templates. This is the only mode supported by installed binaries.
    #[arg(long, conflicts_with = "symlink")]
    pub copy: bool,

    /// Reserved for checkout scripts; installed binaries cannot symlink embedded templates.
    #[arg(long)]
    pub symlink: bool,

    /// Project directory for OpenCode, Cursor, and Copilot adapters.
    #[arg(long)]
    pub project_dir: Option<PathBuf>,

    /// Codex home directory. Defaults to CODEX_HOME or ~/.codex.
    #[arg(long)]
    pub codex_home: Option<PathBuf>,

    /// Claude Code home directory. Defaults to ~/.claude.
    #[arg(long)]
    pub claude_home: Option<PathBuf>,

    /// Gemini CLI home directory. Defaults to ~/.gemini.
    #[arg(long)]
    pub gemini_home: Option<PathBuf>,

    /// Windsurf home directory. Defaults to ~/.codeium/windsurf.
    #[arg(long)]
    pub windsurf_home: Option<PathBuf>,
}
