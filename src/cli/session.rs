use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct SessionCommand {
    #[command(subcommand)]
    pub command: SessionSubcommand,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum SessionSubcommand {
    /// List local session names.
    List,
    /// Import and verify a real-browser session for an authorized page.
    Authorize(AuthorizeSessionCommand),
    /// Inspect one local session with secrets redacted by default.
    Inspect(InspectSessionCommand),
    /// Delete one local session.
    Delete(DeleteSessionCommand),
    /// Import browser/session state from an explicit local source.
    Import(ImportSessionCommand),
    /// Save a deterministic composition of existing sessions.
    Compose(ComposeSessionCommand),
    /// Start, finish, or cancel an experimental user-driven login flow.
    Login(LoginSessionCommand),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BrowserChoice {
    Chrome,
    Chromium,
    Brave,
    Edge,
    Arc,
    Firefox,
    Safari,
}

impl BrowserChoice {
    pub fn as_str(self) -> &'static str {
        match self {
            BrowserChoice::Chrome => "chrome",
            BrowserChoice::Chromium => "chromium",
            BrowserChoice::Brave => "brave",
            BrowserChoice::Edge => "edge",
            BrowserChoice::Arc => "arc",
            BrowserChoice::Firefox => "firefox",
            BrowserChoice::Safari => "safari",
        }
    }
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct AuthorizeSessionCommand {
    /// Local session name to create or replace after import.
    pub name: String,

    /// URL to fetch before and after browser profile import.
    #[arg(long)]
    pub url: String,

    /// Browser source for session import. Chrome is the first supported source.
    #[arg(long, value_enum, default_value_t = BrowserChoice::Chrome)]
    pub browser: BrowserChoice,

    /// Browser profile name or path for the selected browser.
    #[arg(long = "browser-profile")]
    pub browser_profile: Option<String>,

    /// Chrome profile name or path. Compatibility alias for `--browser-profile`.
    #[arg(long = "chrome-profile")]
    pub chrome_profile: Option<String>,

    /// Explicit allowed cookie/storage domain to import. Repeat for each trusted domain.
    #[arg(long = "allow-domain", required = true)]
    pub allow_domain: Vec<String>,

    /// Generic verification predicate that must appear in the session-backed fetch.
    #[arg(long = "must-contain")]
    pub must_contain: Vec<String>,

    /// Generic verification predicate that must be absent in the session-backed fetch.
    #[arg(long = "must-not-contain")]
    pub must_not_contain: Vec<String>,

    /// Write the session-backed verification content to this path.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct LoginSessionCommand {
    #[command(subcommand)]
    pub command: LoginSessionSubcommand,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum LoginSessionSubcommand {
    /// Open a user-driven login browser flow.
    Start(LoginStartCommand),
    /// Save scoped browser state from a pending login flow.
    Finish(LoginFinishCommand),
    /// Cancel a pending login flow and clean up local temp state.
    Cancel(LoginCancelCommand),
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct LoginStartCommand {
    /// Local session name to create when login is finished.
    pub name: String,

    /// Login or target URL to open in the controlled browser profile.
    #[arg(long)]
    pub url: String,

    /// Agent-browser profile name for this experimental login flow.
    #[arg(long)]
    pub profile: Option<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct LoginFinishCommand {
    pub name: String,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct LoginCancelCommand {
    pub name: String,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ComposeSessionCommand {
    /// New local session name to save.
    pub name: String,

    /// Source session name. Repeat to compose several sessions in order.
    #[arg(long, required = true)]
    pub session: Vec<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ImportSessionCommand {
    #[command(subcommand)]
    pub source: ImportSessionSource,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum ImportSessionSource {
    /// Import cookies from a local cmux browser surface.
    Cmux(ImportCmuxSessionCommand),
    /// Import scoped cookies/storage from an explicitly selected browser profile.
    Browser(ImportBrowserSessionCommand),
    /// Import scoped cookies/storage from a Chrome profile snapshot.
    Chrome(ImportChromeSessionCommand),
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ImportCmuxSessionCommand {
    /// cmux browser surface to read cookies from.
    #[arg(long)]
    pub surface: String,

    /// Local session name to create.
    #[arg(long)]
    pub name: String,

    /// Explicit allowed cookie domain to import. Repeat for each trusted domain.
    #[arg(long, required = true)]
    pub allow_domain: Vec<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ImportBrowserSessionCommand {
    /// Browser family to import from. Chrome is the first supported import source.
    #[arg(long, value_enum, default_value_t = BrowserChoice::Chrome)]
    pub browser: BrowserChoice,

    /// Browser profile name or path for the selected browser.
    #[arg(long = "browser-profile")]
    pub browser_profile: Option<String>,

    /// Explicit filesystem path to a browser profile/user-data directory.
    #[arg(long = "profile-path")]
    pub profile_path: Option<PathBuf>,

    /// Local session name to create.
    #[arg(long)]
    pub name: String,

    /// Explicit allowed cookie/storage domain to import. Repeat for each trusted domain.
    #[arg(long, required = true)]
    pub allow_domain: Vec<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ImportChromeSessionCommand {
    /// Chrome profile name or path to snapshot through local Chrome/CDP.
    #[arg(long = "chrome-profile")]
    pub chrome_profile: String,

    /// Local session name to create.
    #[arg(long)]
    pub name: String,

    /// Explicit allowed cookie/storage domain to import. Repeat for each trusted domain.
    #[arg(long, required = true)]
    pub allow_domain: Vec<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct InspectSessionCommand {
    /// Local session name to inspect.
    pub name: String,

    /// Print secret cookie/storage values instead of redactions.
    #[arg(long)]
    pub show_secrets: bool,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct DeleteSessionCommand {
    /// Local session name to delete.
    pub name: String,
}
