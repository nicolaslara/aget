use std::path::PathBuf;

use clap::Args;

use super::{parse_backend_option, ExtractorOption, InlineContent, OutputFormat};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct GetCommand {
    /// HTTP(S), raw:, raw://, or file:// input to extract.
    pub url: String,

    /// Local session name to replay. Repeat to compose sessions for this request.
    #[arg(long)]
    pub session: Vec<String>,

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

    /// Advanced unstable backend escape hatch, e.g. `crawl4ai.wait_for_images=true`.
    #[arg(long = "backend-option", value_parser = parse_backend_option)]
    pub backend_options: Vec<ExtractorOption>,
}
