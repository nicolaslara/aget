use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Args, Parser, Subcommand};

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

        if let Some(first_arg) = args.get(1) {
            let first_arg = first_arg.to_string_lossy();
            if looks_like_url(&first_arg) {
                args.insert(1, OsString::from("get"));
            }
        }

        Self::try_parse_from(args)
    }
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct GlobalOptions {
    #[arg(long, global = true)]
    pub json: bool,

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
    pub session: Option<String>,

    #[arg(long)]
    pub out: Option<PathBuf>,
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

fn parse_duration_secs(value: &str) -> Result<Duration, String> {
    let secs = value
        .parse::<u64>()
        .map_err(|_| format!("expected timeout in seconds, got '{value}'"))?;
    Ok(Duration::from_secs(secs))
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
                session: None,
                out: None,
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
                session: None,
                out: None,
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
                session: None,
                out: Some(PathBuf::from("page.md")),
            })
        );
    }

    #[test]
    fn parses_get_session() {
        let cli = Cli::try_parse_from(["aget", "get", "https://example.com", "--session", "demo"])
            .unwrap();

        assert_eq!(
            cli.command,
            Command::Get(GetCommand {
                url: "https://example.com".to_string(),
                session: Some("demo".to_string()),
                out: None,
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
}
