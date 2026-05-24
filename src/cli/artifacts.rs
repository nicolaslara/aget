use clap::{Args, Subcommand};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ArtifactsCommand {
    #[command(subcommand)]
    pub command: ArtifactsSubcommand,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum ArtifactsSubcommand {
    /// List local run artifacts.
    List,
    /// Inspect one run artifact.
    Inspect(InspectArtifactCommand),
    /// Delete one internal run artifact directory.
    Delete(DeleteArtifactCommand),
    /// Prune internal run artifact directories by explicit retention rules.
    Prune(PruneArtifactsCommand),
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct InspectArtifactCommand {
    /// Run directory id, for example run-123-456.
    pub run_id: String,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct DeleteArtifactCommand {
    /// Run directory id, for example run-123-456.
    pub run_id: String,

    /// Actually delete the internal run directory.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct PruneArtifactsCommand {
    /// Delete runs older than this duration, for example 7d, 12h, or 30m.
    #[arg(long, value_parser = parse_duration_secs)]
    pub older_than: Option<u64>,

    /// Preserve the newest N runs after applying other selectors.
    #[arg(long)]
    pub keep_last: Option<usize>,

    /// Delete oldest runs until internal run storage is at or below this byte count.
    #[arg(long)]
    pub max_bytes: Option<u64>,

    /// Show the prune plan without deleting.
    #[arg(long)]
    pub dry_run: bool,

    /// Actually delete the computed prune plan.
    #[arg(long)]
    pub yes: bool,
}

fn parse_duration_secs(value: &str) -> Result<u64, String> {
    let (number, multiplier) = match value.chars().last() {
        Some('d') => (&value[..value.len() - 1], 24 * 60 * 60),
        Some('h') => (&value[..value.len() - 1], 60 * 60),
        Some('m') => (&value[..value.len() - 1], 60),
        Some('s') => (&value[..value.len() - 1], 1),
        Some(character) if character.is_ascii_digit() => (value, 1),
        _ => {
            return Err(format!(
                "expected duration like 30d, 12h, 30m, or 60s, got '{value}'"
            ))
        }
    };
    let number = number
        .parse::<u64>()
        .map_err(|_| format!("expected duration like 30d, 12h, 30m, or 60s, got '{value}'"))?;
    number
        .checked_mul(multiplier)
        .ok_or_else(|| format!("duration is too large: '{value}'"))
}
