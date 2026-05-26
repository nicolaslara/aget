use clap::Parser;

use super::super::*;

#[test]
fn parses_batch_command() {
    let cli = Cli::try_parse_from([
        "aget",
        "batch",
        "https://example.com/a",
        "https://example.com/b",
        "--session",
        "docs",
        "--content-format",
        "html",
        "--concurrency",
        "3",
        "--fail-fast",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Batch(BatchCommand {
            urls: vec![
                "https://example.com/a".to_string(),
                "https://example.com/b".to_string(),
            ],
            file: None,
            stdin: false,
            session: vec!["docs".to_string()],
            content_format: OutputFormat::Html,
            selector: None,
            exclude_selector: None,
            wait_for_selector: None,
            max_chars: None,
            backend_options: Vec::new(),
            cache: CacheCommandOptions::default(),
            concurrency: 3,
            output_dir: None,
            fail_fast: true,
        })
    );
}
