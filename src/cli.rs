use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser, PartialEq, Eq)]
#[command(name = "aget", version, about = "Local-first agent web context tool")]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOptions,

    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn parse_from_aliasing_get<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let mut args: Vec<OsString> = args.into_iter().map(Into::into).collect();

        if let Some(url_index) = alias_url_index(&args) {
            args.insert(url_index, OsString::from("get"));
        }

        Self::try_parse_from(args)
    }
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct GlobalOptions {
    /// Response envelope format. Use `json` for stable agent/tool output.
    #[arg(long, value_enum, default_value_t = EnvelopeFormat::None, global = true)]
    pub envelope: EnvelopeFormat,

    /// Request timeout in seconds for backend extraction or browser/session actions.
    #[arg(long, value_parser = parse_duration_secs, global = true)]
    pub timeout: Option<Duration>,

    /// Reserve extra diagnostic output for future human-mode debugging.
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Suppress normal human output. Errors and JSON envelopes are still emitted.
    #[arg(long, global = true)]
    pub quiet: bool,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Command {
    /// Extract one HTTP(S) URL into agent-ready content.
    Get(GetCommand),
    /// Manage local auth/session state.
    Session(SessionCommand),
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct GetCommand {
    /// HTTP(S) URL to extract.
    pub url: String,

    /// Local session name to replay. Repeat to compose sessions for this request.
    #[arg(long)]
    pub session: Vec<String>,

    /// Write extracted page content to this path.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Extracted page content format. This is separate from `--envelope`.
    #[arg(long = "content-format", value_enum, default_value_t = OutputFormat::Markdown)]
    pub content_format: OutputFormat,

    /// Whether JSON envelopes include `data.content`.
    #[arg(long = "inline-content", value_enum, default_value_t = InlineContent::Auto)]
    pub inline_content: InlineContent,

    /// CSS selector used to keep only matching page content.
    #[arg(long)]
    pub selector: Option<String>,

    /// CSS selector used to remove matching page content.
    #[arg(long)]
    pub exclude_selector: Option<String>,

    /// CSS selector to wait for before extraction.
    #[arg(long)]
    pub wait_for_selector: Option<String>,

    /// Deterministically truncate extracted content to this many Unicode scalar values.
    #[arg(long)]
    pub max_chars: Option<usize>,

    /// Advanced unstable backend escape hatch, e.g. `crawl4ai.wait_for_images=true`.
    #[arg(long = "backend-option", value_parser = parse_backend_option)]
    pub backend_options: Vec<ExtractorOption>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EnvelopeFormat {
    Json,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InlineContent {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Markdown,
    Html,
    Text,
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            OutputFormat::Markdown => "markdown",
            OutputFormat::Html => "html",
            OutputFormat::Text => "text",
            OutputFormat::Json => "json",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractorOption {
    pub key: String,
    pub value: String,
}

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

fn looks_like_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn alias_url_index(args: &[OsString]) -> Option<usize> {
    let mut saw_command = false;
    for (index, arg) in args.iter().enumerate().skip(1) {
        let arg = arg.to_string_lossy();
        if matches!(arg.as_ref(), "get" | "session") {
            saw_command = true;
        }
        if saw_command {
            return None;
        }
        if looks_like_url(&arg) {
            return Some(index);
        }
    }
    None
}

fn parse_duration_secs(value: &str) -> Result<Duration, String> {
    let secs = value
        .parse::<u64>()
        .map_err(|_| format!("expected timeout in seconds, got '{value}'"))?;
    Ok(Duration::from_secs(secs))
}

fn parse_backend_option(value: &str) -> Result<ExtractorOption, String> {
    let (key, option_value) = value
        .split_once('=')
        .ok_or_else(|| "expected backend option in key=value form".to_string())?;
    if key.is_empty() {
        return Err("backend option key must not be empty".to_string());
    }
    if !key.starts_with("crawl4ai.") {
        return Err("backend option key must be namespaced, e.g. crawl4ai.wait_until".to_string());
    }
    if key.trim_start_matches("crawl4ai.").is_empty() {
        return Err("backend option key must include a crawl4ai option name".to_string());
    }
    Ok(ExtractorOption {
        key: key.to_string(),
        value: option_value.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_get_command() {
        let cli = Cli::try_parse_from(["aget", "get", "https://example.com"]).unwrap();

        assert_eq!(
            cli.command,
            Command::Get(GetCommand {
                url: "https://example.com".to_string(),
                session: Vec::new(),
                output: None,
                content_format: OutputFormat::Markdown,
                inline_content: InlineContent::Auto,
                selector: None,
                exclude_selector: None,
                wait_for_selector: None,
                max_chars: None,
                backend_options: Vec::new(),
            })
        );
    }

    #[test]
    fn aliases_top_level_url_to_get_command() {
        let cli = Cli::parse_from_aliasing_get(["aget", "https://example.com"]).unwrap();

        assert_eq!(
            cli.command,
            Command::Get(GetCommand {
                url: "https://example.com".to_string(),
                session: Vec::new(),
                output: None,
                content_format: OutputFormat::Markdown,
                inline_content: InlineContent::Auto,
                selector: None,
                exclude_selector: None,
                wait_for_selector: None,
                max_chars: None,
                backend_options: Vec::new(),
            })
        );
    }

    #[test]
    fn parses_global_flags_before_subcommand() {
        let cli = Cli::try_parse_from([
            "aget",
            "--envelope",
            "json",
            "--timeout",
            "30",
            "get",
            "https://example.com",
        ])
        .unwrap();

        assert_eq!(cli.global.envelope, EnvelopeFormat::Json);
        assert_eq!(cli.global.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn parses_envelope_for_structured_output() {
        let cli = Cli::try_parse_from(["aget", "get", "https://example.com", "--envelope", "json"])
            .unwrap();

        assert_eq!(cli.global.envelope, EnvelopeFormat::Json);
    }

    #[test]
    fn parses_global_flags_after_subcommand() {
        let cli = Cli::try_parse_from([
            "aget",
            "get",
            "https://example.com",
            "--envelope",
            "json",
            "--quiet",
        ])
        .unwrap();

        assert_eq!(cli.global.envelope, EnvelopeFormat::Json);
        assert!(cli.global.quiet);
    }

    #[test]
    fn parses_get_output_path() {
        let cli =
            Cli::try_parse_from(["aget", "get", "https://example.com", "--output", "page.md"])
                .unwrap();

        assert_eq!(
            cli.command,
            Command::Get(GetCommand {
                url: "https://example.com".to_string(),
                session: Vec::new(),
                output: Some(PathBuf::from("page.md")),
                content_format: OutputFormat::Markdown,
                inline_content: InlineContent::Auto,
                selector: None,
                exclude_selector: None,
                wait_for_selector: None,
                max_chars: None,
                backend_options: Vec::new(),
            })
        );
    }

    #[test]
    fn parses_repeated_get_sessions() {
        let cli = Cli::try_parse_from([
            "aget",
            "get",
            "https://example.com",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Get(GetCommand {
                url: "https://example.com".to_string(),
                session: vec!["provider".to_string(), "app".to_string()],
                output: None,
                content_format: OutputFormat::Markdown,
                inline_content: InlineContent::Auto,
                selector: None,
                exclude_selector: None,
                wait_for_selector: None,
                max_chars: None,
                backend_options: Vec::new(),
            })
        );
    }

    #[test]
    fn parses_get_output_shaping_options() {
        let cli = Cli::try_parse_from([
            "aget",
            "get",
            "https://example.com",
            "--content-format",
            "json",
            "--inline-content",
            "never",
            "--selector",
            "main",
            "--exclude-selector",
            "nav",
            "--wait-for-selector",
            "css:.ready",
            "--max-chars",
            "123",
            "--backend-option",
            "crawl4ai.cache=bypass",
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Get(GetCommand {
                url: "https://example.com".to_string(),
                session: Vec::new(),
                output: None,
                content_format: OutputFormat::Json,
                inline_content: InlineContent::Never,
                selector: Some("main".to_string()),
                exclude_selector: Some("nav".to_string()),
                wait_for_selector: Some("css:.ready".to_string()),
                max_chars: Some(123),
                backend_options: vec![ExtractorOption {
                    key: "crawl4ai.cache".to_string(),
                    value: "bypass".to_string(),
                }],
            })
        );
    }

    #[test]
    fn parses_session_inspect() {
        let cli = Cli::try_parse_from(["aget", "session", "inspect", "demo"]).unwrap();

        assert_eq!(
            cli.command,
            Command::Session(SessionCommand {
                command: SessionSubcommand::Inspect(InspectSessionCommand {
                    name: "demo".to_string(),
                    show_secrets: false
                })
            })
        );
    }

    #[test]
    fn parses_session_authorize_chrome() {
        let cli = Cli::try_parse_from([
            "aget",
            "session",
            "authorize",
            "news",
            "--url",
            "https://example.com/account",
            "--browser",
            "chrome",
            "--browser-profile",
            "Default",
            "--allow-domain",
            "example.com",
            "--must-contain",
            "Welcome",
            "--must-not-contain",
            "Please sign in",
            "--output",
            "verification.md",
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Session(SessionCommand {
                command: SessionSubcommand::Authorize(AuthorizeSessionCommand {
                    name: "news".to_string(),
                    url: "https://example.com/account".to_string(),
                    browser: BrowserChoice::Chrome,
                    browser_profile: Some("Default".to_string()),
                    chrome_profile: None,
                    allow_domain: vec!["example.com".to_string()],
                    must_contain: vec!["Welcome".to_string()],
                    must_not_contain: vec!["Please sign in".to_string()],
                    output: Some(PathBuf::from("verification.md")),
                })
            })
        );
    }

    #[test]
    fn parses_session_authorize_chrome_profile_alias() {
        let cli = Cli::try_parse_from([
            "aget",
            "session",
            "authorize",
            "news",
            "--url",
            "https://example.com/account",
            "--chrome-profile",
            "Default",
            "--allow-domain",
            "example.com",
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Session(SessionCommand {
                command: SessionSubcommand::Authorize(AuthorizeSessionCommand {
                    name: "news".to_string(),
                    url: "https://example.com/account".to_string(),
                    browser: BrowserChoice::Chrome,
                    browser_profile: None,
                    chrome_profile: Some("Default".to_string()),
                    allow_domain: vec!["example.com".to_string()],
                    must_contain: Vec::new(),
                    must_not_contain: Vec::new(),
                    output: None,
                })
            })
        );
    }

    #[test]
    fn parses_session_import_cmux_with_repeated_domains() {
        let cli = Cli::try_parse_from([
            "aget",
            "session",
            "import",
            "cmux",
            "--surface",
            "surface:1",
            "--name",
            "demo",
            "--allow-domain",
            "example.com",
            "--allow-domain",
            "docs.example.com",
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Session(SessionCommand {
                command: SessionSubcommand::Import(ImportSessionCommand {
                    source: ImportSessionSource::Cmux(ImportCmuxSessionCommand {
                        surface: "surface:1".to_string(),
                        name: "demo".to_string(),
                        allow_domain: vec![
                            "example.com".to_string(),
                            "docs.example.com".to_string()
                        ],
                    })
                })
            })
        );
    }

    #[test]
    fn parses_session_compose_with_repeated_sessions() {
        let cli = Cli::try_parse_from([
            "aget",
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Session(SessionCommand {
                command: SessionSubcommand::Compose(ComposeSessionCommand {
                    name: "combined".to_string(),
                    session: vec!["provider".to_string(), "app".to_string()],
                })
            })
        );
    }

    #[test]
    fn parses_session_login_start() {
        let cli = Cli::try_parse_from([
            "aget",
            "session",
            "login",
            "start",
            "news",
            "--url",
            "https://www.nytimes.com/article",
            "--profile",
            "aget-news",
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Session(SessionCommand {
                command: SessionSubcommand::Login(LoginSessionCommand {
                    command: LoginSessionSubcommand::Start(LoginStartCommand {
                        name: "news".to_string(),
                        url: "https://www.nytimes.com/article".to_string(),
                        profile: Some("aget-news".to_string()),
                    })
                })
            })
        );
    }

    #[test]
    fn parses_session_login_finish_and_cancel() {
        let finish = Cli::try_parse_from(["aget", "session", "login", "finish", "news"]).unwrap();
        let cancel = Cli::try_parse_from(["aget", "session", "login", "cancel", "news"]).unwrap();

        assert_eq!(
            finish.command,
            Command::Session(SessionCommand {
                command: SessionSubcommand::Login(LoginSessionCommand {
                    command: LoginSessionSubcommand::Finish(LoginFinishCommand {
                        name: "news".to_string(),
                    })
                })
            })
        );
        assert_eq!(
            cancel.command,
            Command::Session(SessionCommand {
                command: SessionSubcommand::Login(LoginSessionCommand {
                    command: LoginSessionSubcommand::Cancel(LoginCancelCommand {
                        name: "news".to_string(),
                    })
                })
            })
        );
    }

    #[test]
    fn parses_session_import_chrome_with_repeated_domains() {
        let cli = Cli::try_parse_from([
            "aget",
            "session",
            "import",
            "chrome",
            "--chrome-profile",
            "Default",
            "--name",
            "demo",
            "--allow-domain",
            "example.com",
            "--allow-domain",
            "docs.example.com",
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Session(SessionCommand {
                command: SessionSubcommand::Import(ImportSessionCommand {
                    source: ImportSessionSource::Chrome(ImportChromeSessionCommand {
                        chrome_profile: "Default".to_string(),
                        name: "demo".to_string(),
                        allow_domain: vec![
                            "example.com".to_string(),
                            "docs.example.com".to_string()
                        ],
                    })
                })
            })
        );
    }
}
