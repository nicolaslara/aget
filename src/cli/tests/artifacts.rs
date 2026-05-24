use clap::Parser;

use super::super::*;

#[test]
fn parses_artifacts_subcommands() {
    let cli = Cli::try_parse_from(["aget", "artifacts", "inspect", "run-1-2"]).unwrap();
    assert_eq!(
        cli.command,
        Command::Artifacts(ArtifactsCommand {
            command: ArtifactsSubcommand::Inspect(InspectArtifactCommand {
                run_id: "run-1-2".to_string(),
            }),
        })
    );

    let cli = Cli::try_parse_from([
        "aget",
        "artifacts",
        "prune",
        "--older-than",
        "7d",
        "--keep-last",
        "3",
        "--max-bytes",
        "1000",
        "--dry-run",
    ])
    .unwrap();
    assert_eq!(
        cli.command,
        Command::Artifacts(ArtifactsCommand {
            command: ArtifactsSubcommand::Prune(PruneArtifactsCommand {
                older_than: Some(7 * 24 * 60 * 60),
                keep_last: Some(3),
                max_bytes: Some(1000),
                dry_run: true,
                yes: false,
            }),
        })
    );
}
