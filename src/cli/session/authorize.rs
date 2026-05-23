use std::path::PathBuf;

use clap::Args;

use super::BrowserChoice;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct AuthorizeSessionCommand {
    /// Local session name to create or replace after import.
    pub name: String,

    /// URL to fetch before and after browser profile import.
    #[arg(long)]
    pub url: String,

    /// Browser source for session import. Chrome is the first supported source.
    #[arg(long, value_enum, default_value_t = BrowserChoice::Chrome)]
    pub browser: BrowserChoice,

    /// Browser profile name or path for the selected browser.
    #[arg(long = "browser-profile")]
    pub browser_profile: Option<String>,

    /// Chrome profile name or path. Compatibility alias for `--browser-profile`.
    #[arg(long = "chrome-profile")]
    pub chrome_profile: Option<String>,

    /// Explicit allowed cookie/storage domain to import. Repeat for each trusted domain.
    #[arg(long = "allow-domain", required = true)]
    pub allow_domain: Vec<String>,

    /// Generic verification predicate that must appear in the session-backed fetch.
    #[arg(long = "must-contain")]
    pub must_contain: Vec<String>,

    /// Generic verification predicate that must be absent in the session-backed fetch.
    #[arg(long = "must-not-contain")]
    pub must_not_contain: Vec<String>,

    /// Write the session-backed verification content to this path.
    #[arg(long)]
    pub output: Option<PathBuf>,
}
