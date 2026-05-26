use clap::Parser;

use super::super::*;

#[test]
fn parses_crawl_command() {
    let cli = Cli::try_parse_from([
        "aget",
        "crawl",
        "https://example.com/docs/",
        "--limit",
        "10",
        "--max-depth",
        "3",
        "--concurrency",
        "2",
        "--any-path",
        "--allow-domain",
        "cdn.example.com",
        "--include",
        "*/docs/*",
        "--exclude",
        "*/logout",
        "--session",
        "docs",
        "--content-format",
        "html",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Crawl(CrawlCommand {
            url: "https://example.com/docs/".to_string(),
            limit: 10,
            max_depth: 3,
            concurrency: 2,
            any_origin: false,
            same_origin: false,
            any_path: true,
            same_path: false,
            allow_domains: vec!["cdn.example.com".to_string()],
            include: vec!["*/docs/*".to_string()],
            exclude: vec!["*/logout".to_string()],
            session: vec!["docs".to_string()],
            content_format: OutputFormat::Html,
            selector: None,
            exclude_selector: None,
            wait_for_selector: None,
            max_chars: None,
            cache: CacheCommandOptions::default(),
            output_dir: None,
            backend_options: Vec::new(),
        })
    );
}
