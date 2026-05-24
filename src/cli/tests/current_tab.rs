use clap::Parser;

use super::super::*;

#[test]
fn parses_current_tab_command() {
    let cli = Cli::try_parse_from([
        "aget",
        "current-tab",
        "--cdp-port",
        "9222",
        "--allow-private-content",
        "--content-format",
        "text",
        "--inline-content",
        "never",
        "--selector",
        "main",
        "--exclude-selector",
        "nav",
        "--wait-for-selector",
        ".ready",
        "--max-chars",
        "456",
        "--backend-option",
        "aget.wait_for_images=true",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::CurrentTab(CurrentTabCommand {
            cdp_port: 9222,
            allow_private_content: true,
            output: None,
            content_format: OutputFormat::Text,
            inline_content: InlineContent::Never,
            selector: Some("main".to_string()),
            exclude_selector: Some("nav".to_string()),
            wait_for_selector: Some(".ready".to_string()),
            max_chars: Some(456),
            backend_options: vec![ExtractorOption {
                key: "aget.wait_for_images".to_string(),
                value: "true".to_string(),
            }],
        })
    );
}
