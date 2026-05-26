use clap::Parser;

use super::super::*;

#[test]
fn parses_map_command() {
    let cli = Cli::try_parse_from([
        "aget",
        "map",
        "https://example.com/docs/intro",
        "--session",
        "docs",
        "--any-origin",
        "--include",
        "*/docs/*",
        "--exclude",
        "*/logout",
        "--content-type",
        "text/html",
        "--max-links",
        "20",
        "--output",
        "json",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Map(MapCommand {
            url: Some("https://example.com/docs/intro".to_string()),
            artifact: None,
            session: vec!["docs".to_string()],
            selector: None,
            exclude_selector: None,
            wait_for_selector: None,
            any_origin: true,
            same_origin: false,
            any_path: false,
            same_path: false,
            include: vec!["*/docs/*".to_string()],
            exclude: vec!["*/logout".to_string()],
            max_links: 20,
            cache: CacheCommandOptions::default(),
            content_types: vec!["text/html".to_string()],
            output: MapOutput::Json,
            backend_options: Vec::new(),
        })
    );
}
