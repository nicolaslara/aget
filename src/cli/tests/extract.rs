use std::path::PathBuf;

use clap::Parser;

use super::super::*;

#[test]
fn parses_extract_artifact_command() {
    let cli = Cli::try_parse_from([
        "aget",
        "extract",
        "--artifact",
        "run-123",
        "--field",
        "tables",
        "--field",
        "links",
        "--schema",
        "/tmp/schema.json",
        "--allow-private-content",
        "--output",
        "markdown",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Extract(ExtractCommand {
            artifact: Some("run-123".to_string()),
            manifest: None,
            schema: Some(PathBuf::from("/tmp/schema.json")),
            fields: vec!["tables".to_string(), "links".to_string()],
            allow_private_content: true,
            output: ExtractOutput::Markdown,
        })
    );
}

#[test]
fn parses_extract_manifest_command() {
    let cli = Cli::try_parse_from([
        "aget",
        "extract",
        "--manifest",
        "/tmp/crawl/manifest.json",
        "--field",
        "headings",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Extract(ExtractCommand {
            artifact: None,
            manifest: Some(PathBuf::from("/tmp/crawl/manifest.json")),
            schema: None,
            fields: vec!["headings".to_string()],
            allow_private_content: false,
            output: ExtractOutput::Json,
        })
    );
}
