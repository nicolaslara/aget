use std::path::PathBuf;

use clap::Parser;

use super::super::*;

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
            cache: CacheCommandOptions::default(),
            backend_options: Vec::new(),
        })
    );
}

#[test]
fn parses_get_output_path() {
    let cli =
        Cli::try_parse_from(["aget", "get", "https://example.com", "--output", "page.md"]).unwrap();

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
            cache: CacheCommandOptions::default(),
            backend_options: Vec::new(),
        })
    );
}

#[test]
fn parses_get_local_content_input() {
    let cli = Cli::try_parse_from(["aget", "get", "raw:<main>Local</main>"]).unwrap();

    assert_eq!(
        cli.command,
        Command::Get(GetCommand {
            url: "raw:<main>Local</main>".to_string(),
            session: Vec::new(),
            output: None,
            content_format: OutputFormat::Markdown,
            inline_content: InlineContent::Auto,
            selector: None,
            exclude_selector: None,
            wait_for_selector: None,
            max_chars: None,
            cache: CacheCommandOptions::default(),
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
            cache: CacheCommandOptions::default(),
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
        "aget.cache=bypass",
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
            cache: CacheCommandOptions::default(),
            backend_options: vec![ExtractorOption {
                key: "aget.cache".to_string(),
                value: "bypass".to_string(),
            }],
        })
    );
}

#[test]
fn parses_get_cache_controls() {
    let cli = Cli::try_parse_from([
        "aget",
        "get",
        "https://example.com",
        "--fresh",
        "--cache-ttl",
        "120",
    ])
    .unwrap();

    let Command::Get(get) = cli.command else {
        panic!("expected get command");
    };
    assert!(get.cache.fresh);
    assert_eq!(get.cache.policy(), CachePolicy::Refresh);
    assert_eq!(get.cache.cache_ttl, std::time::Duration::from_secs(120));

    let cli = Cli::try_parse_from([
        "aget",
        "get",
        "https://example.com",
        "--cache-policy",
        "off",
    ])
    .unwrap();
    let Command::Get(get) = cli.command else {
        panic!("expected get command");
    };
    assert!(!get.cache.fresh);
    assert_eq!(get.cache.policy(), CachePolicy::Off);
}

#[test]
fn rejects_conflicting_cache_controls() {
    let error = Cli::try_parse_from([
        "aget",
        "get",
        "https://example.com",
        "--fresh",
        "--cache-policy",
        "off",
    ])
    .unwrap_err();

    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
}
