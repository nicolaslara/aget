use std::path::PathBuf;

use clap::Args;

use super::{parse_backend_option, ExtractorOption, OutputFormat};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct CrawlCommand {
    /// Start URL for bounded traversal.
    pub url: String,

    /// Maximum number of pages to fetch. Required.
    #[arg(long)]
    pub limit: usize,

    /// Maximum link depth from the start URL.
    #[arg(long, default_value_t = 2)]
    pub max_depth: usize,

    /// Maximum number of pages to fetch concurrently.
    #[arg(long, default_value_t = 2)]
    pub concurrency: usize,

    /// Allow traversal outside the start origin.
    #[arg(long, conflicts_with = "same_origin")]
    pub any_origin: bool,

    /// Keep traversal on the start origin. This is the default.
    #[arg(long, conflicts_with = "any_origin")]
    pub same_origin: bool,

    /// Allow traversal outside the start path prefix.
    #[arg(long, conflicts_with = "same_path")]
    pub any_path: bool,

    /// Keep traversal under the start path prefix. This is the default.
    #[arg(long, conflicts_with = "any_path")]
    pub same_path: bool,

    /// Additional domain that traversal may visit when same-origin is widened.
    #[arg(long = "allow-domain")]
    pub allow_domains: Vec<String>,

    /// Include only links matching this simple glob pattern. Repeatable.
    #[arg(long)]
    pub include: Vec<String>,

    /// Exclude links matching this simple glob pattern. Repeatable.
    #[arg(long)]
    pub exclude: Vec<String>,

    /// Local session name to replay for each fetched page.
    #[arg(long)]
    pub session: Vec<String>,

    /// Extracted page content format for stored page artifacts.
    #[arg(long = "content-format", value_enum, default_value_t = OutputFormat::Markdown)]
    pub content_format: OutputFormat,

    /// CSS selector used to keep only matching page content.
    #[arg(long)]
    pub selector: Option<String>,

    /// CSS selector used to remove matching page content.
    #[arg(long)]
    pub exclude_selector: Option<String>,

    /// CSS selector to wait for before extraction.
    #[arg(long)]
    pub wait_for_selector: Option<String>,

    /// Deterministically truncate each extracted content item to this many characters.
    #[arg(long)]
    pub max_chars: Option<usize>,

    /// Write crawl manifest and per-page artifacts to this directory.
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Advanced unstable backend option in key=value form.
    #[arg(long = "backend-option", value_parser = parse_backend_option)]
    pub backend_options: Vec<ExtractorOption>,
}
