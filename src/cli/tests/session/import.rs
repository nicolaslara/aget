use std::path::PathBuf;

use clap::Parser;

use super::super::super::*;

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
