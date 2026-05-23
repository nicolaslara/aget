use clap::{Args, Subcommand};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct LoginSessionCommand {
    #[command(subcommand)]
    pub command: LoginSessionSubcommand,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum LoginSessionSubcommand {
    /// Open a user-driven login browser flow.
    Start(LoginStartCommand),
    /// Save scoped browser state from a pending login flow.
    Finish(LoginFinishCommand),
    /// Cancel a pending login flow and clean up local temp state.
    Cancel(LoginCancelCommand),
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct LoginStartCommand {
    /// Local session name to create when login is finished.
    pub name: String,

    /// Login or target URL to open in the controlled browser profile.
    #[arg(long)]
    pub url: String,

    /// Agent-browser profile name for this experimental login flow.
    #[arg(long)]
    pub profile: Option<String>,

    /// Existing local session to inject into the login browser profile. Repeat to inject several.
    #[arg(long = "session")]
    pub session: Vec<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct LoginFinishCommand {
    pub name: String,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct LoginCancelCommand {
    pub name: String,
}
