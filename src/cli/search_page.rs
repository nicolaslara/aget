use clap::{Args, ValueEnum};

#[derive(Debug, Args, PartialEq, Eq)]
pub struct SearchPageCommand {
    /// Internal aget get run id to search.
    #[arg(long)]
    pub artifact: String,

    /// Keyword or objective text to search for.
    #[arg(long)]
    pub query: String,

    /// Maximum snippets to emit.
    #[arg(long = "max-results", default_value_t = 8)]
    pub max_results: usize,

    /// Approximate characters to keep around each match.
    #[arg(long = "context-chars", default_value_t = 240)]
    pub context_chars: usize,

    /// Allow snippets from artifacts marked sensitive.
    #[arg(long = "allow-private-content")]
    pub allow_private_content: bool,

    /// Human output format when --envelope json is not used.
    #[arg(long, value_enum, default_value_t = SearchPageOutput::Markdown)]
    pub output: SearchPageOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SearchPageOutput {
    Markdown,
    Json,
}
