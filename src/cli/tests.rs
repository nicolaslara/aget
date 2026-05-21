use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;

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
                    allow_domain: vec!["example.com".to_string(), "docs.example.com".to_string()],
                })
            })
        })
    );
}

#[test]
fn parses_session_import_browser_with_browser_profile() {
    let cli = Cli::try_parse_from([
        "aget",
        "session",
        "import",
        "browser",
        "--browser",
        "chrome",
        "--browser-profile",
        "Default",
        "--name",
        "demo",
        "--allow-domain",
        "example.com",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Session(SessionCommand {
            command: SessionSubcommand::Import(ImportSessionCommand {
                source: ImportSessionSource::Browser(ImportBrowserSessionCommand {
                    browser: BrowserChoice::Chrome,
                    browser_profile: Some("Default".to_string()),
                    profile_path: None,
                    name: "demo".to_string(),
                    allow_domain: vec!["example.com".to_string()],
                })
            })
        })
    );
}

#[test]
fn parses_session_import_browser_with_profile_path_and_unsupported_family() {
    let cli = Cli::try_parse_from([
        "aget",
        "session",
        "import",
        "browser",
        "--browser",
        "firefox",
        "--profile-path",
        "/tmp/firefox-profile",
        "--name",
        "demo",
        "--allow-domain",
        "example.com",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Session(SessionCommand {
            command: SessionSubcommand::Import(ImportSessionCommand {
                source: ImportSessionSource::Browser(ImportBrowserSessionCommand {
                    browser: BrowserChoice::Firefox,
                    browser_profile: None,
                    profile_path: Some(PathBuf::from("/tmp/firefox-profile")),
                    name: "demo".to_string(),
                    allow_domain: vec!["example.com".to_string()],
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
                    allow_domain: vec!["example.com".to_string(), "docs.example.com".to_string()],
                })
            })
        })
    );
}
