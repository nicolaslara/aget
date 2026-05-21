use std::time::Duration;

use clap::Parser;

use super::super::*;

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
    let cli =
        Cli::try_parse_from(["aget", "get", "https://example.com", "--envelope", "json"]).unwrap();

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
