use std::path::PathBuf;

use clap::Args;

use super::{parse_backend_option, ExtractorOption, OutputFormat};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct BatchCommand {
    /// Explicit URL inputs to fetch.
    pub urls: Vec<String>,

    /// Read newline-delimited URLs from this file.
    #[arg(long)]
    pub file: Option<PathBuf>,

    /// Read newline-delimited URLs from standard input.
    #[arg(long)]
    pub stdin: bool,

    /// Local session name to replay. Repeat to compose sessions for each request.
    #[arg(long)]
    pub session: Vec<String>,

    /// Extracted page content format for each item.
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

    /// Advanced unstable backend option in key=value form.
    #[arg(long = "backend-option", value_parser = parse_backend_option)]
    pub backend_options: Vec<ExtractorOption>,

    /// Maximum number of items to fetch concurrently.
    #[arg(long, default_value_t = 2)]
    pub concurrency: usize,

    /// Write batch manifest and per-item artifacts to this directory.
    #[arg(long)]
    pub output_dir: Option<PathBuf>,

    /// Stop scheduling new items after the first failure.
    #[arg(long)]
    pub fail_fast: bool,
}
