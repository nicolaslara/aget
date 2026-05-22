use clap::Parser;

use super::super::super::*;

#[test]
fn parses_session_login_start() {
    let cli = Cli::try_parse_from([
        "aget",
        "session",
        "login",
        "start",
        "news",
        "--url",
        "https://www.nytimes.com/article",
        "--profile",
        "aget-news",
        "--session",
        "oauth",
        "--session",
        "okta",
    ])
    .unwrap();

    assert_eq!(
        cli.command,
        Command::Session(SessionCommand {
            command: SessionSubcommand::Login(LoginSessionCommand {
                command: LoginSessionSubcommand::Start(LoginStartCommand {
                    name: "news".to_string(),
                    url: "https://www.nytimes.com/article".to_string(),
                    profile: Some("aget-news".to_string()),
                    session: vec!["oauth".to_string(), "okta".to_string()],
                })
            })
        })
    );
}

#[test]
fn parses_session_login_finish_and_cancel() {
    let finish = Cli::try_parse_from(["aget", "session", "login", "finish", "news"]).unwrap();
    let cancel = Cli::try_parse_from(["aget", "session", "login", "cancel", "news"]).unwrap();

    assert_eq!(
        finish.command,
        Command::Session(SessionCommand {
            command: SessionSubcommand::Login(LoginSessionCommand {
                command: LoginSessionSubcommand::Finish(LoginFinishCommand {
                    name: "news".to_string(),
                })
            })
        })
    );
    assert_eq!(
        cancel.command,
        Command::Session(SessionCommand {
            command: SessionSubcommand::Login(LoginSessionCommand {
                command: LoginSessionSubcommand::Cancel(LoginCancelCommand {
                    name: "news".to_string(),
                })
            })
        })
    );
}
