use std::path::PathBuf;

use clap::Parser;

use super::super::*;

#[test]
fn parses_interact_command() {
    let cli = Cli::try_parse_from([
        "aget",
        "interact",
        "https://example.com/app",
        "--actions",
        "actions.json",
        "--allow-actions",
        "--allow-private-content",
        "--allow-sensitive-input",
        "--allow-submit",
        "--capture-screenshot",
        "--session",
        "app",
        "--dry-run",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Interact(InteractCommand {
            source: "https://example.com/app".to_string(),
            actions: PathBuf::from("actions.json"),
            allow_actions: true,
            allow_private_content: true,
            allow_sensitive_input: true,
            allow_submit: true,
            capture_screenshot: true,
            cdp_port: None,
            session: vec!["app".to_string()],
            dry_run: true,
        })
    );
}

#[test]
fn parses_interact_current_tab_port() {
    let cli = Cli::try_parse_from([
        "aget",
        "interact",
        "current-tab",
        "--cdp-port",
        "9222",
        "--actions",
        "actions.json",
        "--allow-actions",
        "--allow-private-content",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Interact(InteractCommand {
            source: "current-tab".to_string(),
            actions: PathBuf::from("actions.json"),
            allow_actions: true,
            allow_private_content: true,
            allow_sensitive_input: false,
            allow_submit: false,
            capture_screenshot: false,
            cdp_port: Some(9222),
            session: Vec::new(),
            dry_run: false,
        })
    );
}
