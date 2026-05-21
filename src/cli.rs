use std::ffi::OsString;
use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};

mod get;
mod session;
#[cfg(test)]
mod tests;

pub use get::GetCommand;
pub use session::{
    AuthorizeSessionCommand, BrowserChoice, ComposeSessionCommand, DeleteSessionCommand,
    ImportBrowserSessionCommand, ImportChromeSessionCommand, ImportCmuxSessionCommand,
    ImportSessionCommand, ImportSessionSource, InspectSessionCommand, LoginCancelCommand,
    LoginFinishCommand, LoginSessionCommand, LoginSessionSubcommand, LoginStartCommand,
    SessionCommand, SessionSubcommand,
};

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
