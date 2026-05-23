use clap::Args;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ComposeSessionCommand {
    /// New local session name to save.
    pub name: String,

    /// Source session name. Repeat to compose several sessions in order.
    #[arg(long, required = true)]
    pub session: Vec<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct InspectSessionCommand {
    /// Local session name to inspect.
    pub name: String,

    /// Print secret cookie/storage values instead of redactions.
    #[arg(long)]
    pub show_secrets: bool,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct DeleteSessionCommand {
    /// Local session name to delete.
    pub name: String,
}
