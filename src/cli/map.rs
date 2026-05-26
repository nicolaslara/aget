use clap::{Args, ValueEnum};

use super::{parse_backend_option, CacheCommandOptions, ExtractorOption};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct MapCommand {
    /// URL input to fetch and map.
    pub url: Option<String>,

    /// Read an existing internal get run artifact by run id.
    #[arg(long)]
    pub artifact: Option<String>,

    /// Local session name to replay when fetching URL input.
    #[arg(long)]
    pub session: Vec<String>,

    /// CSS selector used to keep only matching page content before link extraction.
    #[arg(long)]
    pub selector: Option<String>,

    /// CSS selector used to remove matching page content before link extraction.
    #[arg(long)]
    pub exclude_selector: Option<String>,

    /// CSS selector to wait for before extraction.
    #[arg(long)]
    pub wait_for_selector: Option<String>,

    /// Allow links outside the source origin.
    #[arg(long, conflicts_with = "same_origin")]
    pub any_origin: bool,

    /// Keep only links on the source origin. This is the default.
    #[arg(long, conflicts_with = "any_origin")]
    pub same_origin: bool,

    /// Allow links outside the source path prefix.
    #[arg(long, conflicts_with = "same_path")]
    pub any_path: bool,

    /// Keep only links under the source path prefix. This is the default.
    #[arg(long, conflicts_with = "any_path")]
    pub same_path: bool,

    /// Include only links matching this simple glob pattern. Repeatable.
    #[arg(long)]
    pub include: Vec<String>,

    /// Exclude links matching this simple glob pattern. Repeatable.
    #[arg(long)]
    pub exclude: Vec<String>,

    /// Maximum number of links to emit.
    #[arg(long, default_value_t = 500)]
    pub max_links: usize,

    #[command(flatten)]
    pub cache: CacheCommandOptions,

    /// Keep only links with an inferred content type such as text/html or application/pdf.
    #[arg(long = "content-type")]
    pub content_types: Vec<String>,

    /// Human output format when --envelope json is not used.
    #[arg(long, value_enum, default_value_t = MapOutput::Markdown)]
    pub output: MapOutput,

    /// Advanced unstable backend option in key=value form.
    #[arg(long = "backend-option", value_parser = parse_backend_option)]
    pub backend_options: Vec<ExtractorOption>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MapOutput {
    Markdown,
    Json,
}
