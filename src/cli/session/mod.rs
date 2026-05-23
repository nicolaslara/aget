use clap::{Args, Subcommand};

mod authorize;
mod basic;
mod browser;
mod import;
mod login;

pub use self::authorize::AuthorizeSessionCommand;
pub use self::basic::{ComposeSessionCommand, DeleteSessionCommand, InspectSessionCommand};
pub use self::browser::BrowserChoice;
pub use self::import::{
    ImportBrowserSessionCommand, ImportChromeSessionCommand, ImportCmuxSessionCommand,
    ImportSessionCommand, ImportSessionSource,
};
pub use self::login::{
    LoginCancelCommand, LoginFinishCommand, LoginSessionCommand, LoginSessionSubcommand,
    LoginStartCommand,
};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct SessionCommand {
    #[command(subcommand)]
    pub command: SessionSubcommand,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum SessionSubcommand {
    /// List local session names.
    List,
    /// Import and verify a real-browser session for an authorized page.
    Authorize(AuthorizeSessionCommand),
    /// Inspect one local session with secrets redacted by default.
    Inspect(InspectSessionCommand),
    /// Delete one local session.
    Delete(DeleteSessionCommand),
    /// Import browser/session state from an explicit local source.
    Import(ImportSessionCommand),
    /// Save a deterministic composition of existing sessions.
    Compose(ComposeSessionCommand),
    /// Start, finish, or cancel an experimental user-driven login flow.
    Login(LoginSessionCommand),
}
