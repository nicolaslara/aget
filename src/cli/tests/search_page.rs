use clap::Parser;

use super::super::*;

#[test]
fn parses_search_page_command() {
    let cli = Cli::try_parse_from([
        "aget",
        "search-page",
        "--artifact",
        "run-123",
        "--query",
        "install instructions",
        "--max-results",
        "5",
        "--context-chars",
        "120",
        "--allow-private-content",
        "--output",
        "json",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::SearchPage(SearchPageCommand {
            artifact: "run-123".to_string(),
            query: "install instructions".to_string(),
            max_results: 5,
            context_chars: 120,
            allow_private_content: true,
            output: SearchPageOutput::Json,
        })
    );
}
