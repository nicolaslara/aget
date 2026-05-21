use aget::{ImportSessionSource, LoginSessionSubcommand, SessionSubcommand};

pub(super) fn session_command_name(command: &SessionSubcommand) -> &'static str {
    match command {
        SessionSubcommand::List => "session.list",
        SessionSubcommand::Authorize(_) => "session.authorize",
        SessionSubcommand::Inspect(_) => "session.inspect",
        SessionSubcommand::Delete(_) => "session.delete",
        SessionSubcommand::Import(import) => match &import.source {
            ImportSessionSource::Cmux(_) => "session.import.cmux",
            ImportSessionSource::Browser(_) => "session.import.browser",
            ImportSessionSource::Chrome(_) => "session.import.chrome",
        },
        SessionSubcommand::Compose(_) => "session.compose",
        SessionSubcommand::Login(login) => match &login.command {
            LoginSessionSubcommand::Start(_) => "session.login.start",
            LoginSessionSubcommand::Finish(_) => "session.login.finish",
            LoginSessionSubcommand::Cancel(_) => "session.login.cancel",
        },
    }
}
