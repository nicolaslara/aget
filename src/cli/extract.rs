use std::path::PathBuf;

use clap::{Args, ValueEnum};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ExtractCommand {
    /// Internal aget get run id to extract from.
    #[arg(long, conflicts_with = "manifest")]
    pub artifact: Option<String>,

    /// Batch or crawl manifest JSON path to extract from.
    #[arg(long, conflicts_with = "artifact")]
    pub manifest: Option<PathBuf>,

    /// JSON extraction schema path.
    #[arg(long)]
    pub schema: Option<PathBuf>,

    /// Deterministic built-in field to extract: headings, links, tables, definitions, metadata.
    #[arg(long = "field")]
    pub fields: Vec<String>,

    /// Allow structured values from artifacts marked sensitive.
    #[arg(long = "allow-private-content")]
    pub allow_private_content: bool,

    /// Human output format when --envelope json is not used.
    #[arg(long, value_enum, default_value_t = ExtractOutput::Json)]
    pub output: ExtractOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ExtractOutput {
    Json,
    Markdown,
}
