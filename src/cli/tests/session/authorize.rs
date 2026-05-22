use std::path::PathBuf;

use clap::Parser;

use super::super::super::*;

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
