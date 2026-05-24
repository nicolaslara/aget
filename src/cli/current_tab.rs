use std::path::PathBuf;

use clap::Args;

use super::{parse_backend_option, ExtractorOption, InlineContent, OutputFormat};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct CurrentTabCommand {
    /// Local Chrome DevTools debugging port to read.
    #[arg(long = "cdp-port")]
    pub cdp_port: u16,

    /// Acknowledge that the selected browser tab may contain private content.
    #[arg(long = "allow-private-content")]
    pub allow_private_content: bool,

    /// Write extracted page content to this path.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Extracted page content format. This is separate from `--envelope`.
    #[arg(long = "content-format", value_enum, default_value_t = OutputFormat::Markdown)]
    pub content_format: OutputFormat,

    /// Whether JSON envelopes include `data.content`.
    #[arg(long = "inline-content", value_enum, default_value_t = InlineContent::Auto)]
    pub inline_content: InlineContent,

    /// CSS selector used to keep only matching page content.
    #[arg(long)]
    pub selector: Option<String>,

    /// CSS selector used to remove matching page content.
    #[arg(long)]
    pub exclude_selector: Option<String>,

    /// CSS selector to wait for before extraction.
    #[arg(long)]
    pub wait_for_selector: Option<String>,

    /// Deterministically truncate extracted content to this many Unicode scalar values.
    #[arg(long)]
    pub max_chars: Option<usize>,

    /// Advanced unstable backend option in key=value form.
    #[arg(long = "backend-option", value_parser = parse_backend_option)]
    pub backend_options: Vec<ExtractorOption>,
}
