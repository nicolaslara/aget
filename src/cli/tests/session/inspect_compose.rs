use clap::Parser;

use super::super::super::*;

#[test]
fn parses_session_inspect() {
    let cli = Cli::try_parse_from(["aget", "session", "inspect", "demo"]).unwrap();

    assert_eq!(
        cli.command,
        Command::Session(SessionCommand {
            command: SessionSubcommand::Inspect(InspectSessionCommand {
                name: "demo".to_string(),
                show_secrets: false
            })
        })
    );
}

#[test]
fn parses_session_compose_with_repeated_sessions() {
    let cli = Cli::try_parse_from([
        "aget",
        "session",
        "compose",
        "combined",
        "--session",
        "provider",
        "--session",
        "app",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Session(SessionCommand {
            command: SessionSubcommand::Compose(ComposeSessionCommand {
                name: "combined".to_string(),
                session: vec!["provider".to_string(), "app".to_string()],
            })
        })
    );
}
