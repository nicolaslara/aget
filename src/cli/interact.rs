use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct InteractCommand {
    /// URL to open, or `current-tab` for an explicitly selected local browser tab.
    pub source: String,

    /// Local JSON action plan file.
    #[arg(long)]
    pub actions: PathBuf,

    /// Acknowledge that browser actions can mutate page state.
    #[arg(long = "allow-actions")]
    pub allow_actions: bool,

    /// Acknowledge that sessions or current-tab actions may read private content.
    #[arg(long = "allow-private-content")]
    pub allow_private_content: bool,

    /// Permit action plans marked with sensitive typed input.
    #[arg(long = "allow-sensitive-input")]
    pub allow_sensitive_input: bool,

    /// Permit confirmed submit actions.
    #[arg(long = "allow-submit")]
    pub allow_submit: bool,

    /// Permit capture actions that request screenshots.
    #[arg(long = "capture-screenshot")]
    pub capture_screenshot: bool,

    /// Chrome DevTools debugging port when source is `current-tab`.
    #[arg(long = "cdp-port")]
    pub cdp_port: Option<u16>,

    /// Local session name to replay. Repeat to compose sessions for this request.
    #[arg(long)]
    pub session: Vec<String>,

    /// Validate the plan and write audit artifacts without executing browser actions.
    #[arg(long = "dry-run")]
    pub dry_run: bool,
}
