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
    #[arg(long, global = true)]
    pub json: bool,

    #[arg(long, global = true)]
    pub envelope: bool,

    #[arg(long, value_parser = parse_duration_secs, global = true)]
    pub timeout: Option<Duration>,

    #[arg(long, global = true)]
    pub verbose: bool,

    #[arg(long, global = true)]
    pub quiet: bool,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Command {
    Get(GetCommand),
    Session(SessionCommand),
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct GetCommand {
    pub url: String,

    #[arg(long)]
    pub session: Vec<String>,

    #[arg(long)]
    pub out: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = OutputFormat::Markdown)]
    pub format: OutputFormat,

    #[arg(long)]
    pub selector: Option<String>,

    #[arg(long)]
    pub exclude_selector: Option<String>,

    #[arg(long)]
    pub wait_for: Option<String>,

    #[arg(long)]
    pub max_chars: Option<usize>,

    #[arg(long = "extractor-option", value_parser = parse_extractor_option)]
    pub extractor_options: Vec<ExtractorOption>,
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
    List,
    Inspect(InspectSessionCommand),
    Delete(DeleteSessionCommand),
    Import(ImportSessionCommand),
    Compose(ComposeSessionCommand),
    Login(LoginSessionCommand),
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct LoginSessionCommand {
    #[command(subcommand)]
    pub command: LoginSessionSubcommand,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum LoginSessionSubcommand {
    Start(LoginStartCommand),
    Finish(LoginFinishCommand),
    Cancel(LoginCancelCommand),
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct LoginStartCommand {
    pub name: String,

    #[arg(long)]
    pub url: String,

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
    pub name: String,

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
    Cmux(ImportCmuxSessionCommand),
    Chrome(ImportChromeSessionCommand),
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ImportCmuxSessionCommand {
    #[arg(long)]
    pub surface: String,

    #[arg(long)]
    pub name: String,

    #[arg(long, required = true)]
    pub domain: Vec<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ImportChromeSessionCommand {
    #[arg(long)]
    pub profile: String,

    #[arg(long)]
    pub name: String,

    #[arg(long, required = true)]
    pub domain: Vec<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct InspectSessionCommand {
    pub name: String,

    #[arg(long)]
    pub show_secrets: bool,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct DeleteSessionCommand {
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

fn parse_extractor_option(value: &str) -> Result<ExtractorOption, String> {
    let (key, option_value) = value
        .split_once('=')
        .ok_or_else(|| "expected extractor option in key=value form".to_string())?;
    if key.is_empty() {
        return Err("extractor option key must not be empty".to_string());
    }
    if !key.starts_with("crawl4ai.") {
        return Err(
            "extractor option key must be namespaced, e.g. crawl4ai.wait_until".to_string(),
        );
    }
    if key.trim_start_matches("crawl4ai.").is_empty() {
        return Err("extractor option key must include a crawl4ai option name".to_string());
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
                out: None,
                format: OutputFormat::Markdown,
                selector: None,
                exclude_selector: None,
                wait_for: None,
                max_chars: None,
                extractor_options: Vec::new(),
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
                out: None,
                format: OutputFormat::Markdown,
                selector: None,
                exclude_selector: None,
                wait_for: None,
                max_chars: None,
                extractor_options: Vec::new(),
            })
        );
    }

    #[test]
    fn parses_global_flags_before_subcommand() {
        let cli = Cli::try_parse_from([
            "aget",
            "--json",
            "--timeout",
            "30",
            "get",
            "https://example.com",
        ])
        .unwrap();

        assert!(cli.global.json);
        assert_eq!(cli.global.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn parses_envelope_alias_for_structured_output() {
        let cli =
            Cli::try_parse_from(["aget", "get", "https://example.com", "--envelope"]).unwrap();

        assert!(cli.global.envelope);
    }

    #[test]
    fn parses_global_flags_after_subcommand() {
        let cli = Cli::try_parse_from(["aget", "get", "https://example.com", "--json", "--quiet"])
            .unwrap();

        assert!(cli.global.json);
        assert!(cli.global.quiet);
    }

    #[test]
    fn parses_get_out_path() {
        let cli = Cli::try_parse_from(["aget", "get", "https://example.com", "--out", "page.md"])
            .unwrap();

        assert_eq!(
            cli.command,
            Command::Get(GetCommand {
                url: "https://example.com".to_string(),
                session: Vec::new(),
                out: Some(PathBuf::from("page.md")),
                format: OutputFormat::Markdown,
                selector: None,
                exclude_selector: None,
                wait_for: None,
                max_chars: None,
                extractor_options: Vec::new(),
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
                out: None,
                format: OutputFormat::Markdown,
                selector: None,
                exclude_selector: None,
                wait_for: None,
                max_chars: None,
                extractor_options: Vec::new(),
            })
        );
    }

    #[test]
    fn parses_get_output_shaping_options() {
        let cli = Cli::try_parse_from([
            "aget",
            "get",
            "https://example.com",
            "--format",
            "json",
            "--selector",
            "main",
            "--exclude-selector",
            "nav",
            "--wait-for",
            "css:.ready",
            "--max-chars",
            "123",
            "--extractor-option",
            "crawl4ai.cache=bypass",
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Get(GetCommand {
                url: "https://example.com".to_string(),
                session: Vec::new(),
                out: None,
                format: OutputFormat::Json,
                selector: Some("main".to_string()),
                exclude_selector: Some("nav".to_string()),
                wait_for: Some("css:.ready".to_string()),
                max_chars: Some(123),
                extractor_options: vec![ExtractorOption {
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
            "--domain",
            "example.com",
            "--domain",
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
                        domain: vec!["example.com".to_string(), "docs.example.com".to_string()],
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
            "--profile",
            "Default",
            "--name",
            "demo",
            "--domain",
            "example.com",
            "--domain",
            "docs.example.com",
        ])
        .unwrap();

        assert_eq!(
            cli.command,
            Command::Session(SessionCommand {
                command: SessionSubcommand::Import(ImportSessionCommand {
                    source: ImportSessionSource::Chrome(ImportChromeSessionCommand {
                        profile: "Default".to_string(),
                        name: "demo".to_string(),
                        domain: vec!["example.com".to_string(), "docs.example.com".to_string()],
                    })
                })
            })
        );
    }
}
