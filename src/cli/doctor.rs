use clap::{Args, ValueEnum};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct DoctorCommand {
    /// Skip slower probes such as run-artifact sizing.
    #[arg(long)]
    pub quick: bool,

    /// Run only selected check groups.
    #[arg(long = "check", value_enum)]
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum DoctorCheck {
    Binary,
    Store,
    Artifacts,
    Chrome,
    CurrentTab,
    Cmux,
    Opencode,
}
