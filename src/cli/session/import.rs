use std::path::PathBuf;

use clap::{Args, Subcommand};

use super::BrowserChoice;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ImportSessionCommand {
    #[command(subcommand)]
    pub source: ImportSessionSource,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum ImportSessionSource {
    /// Import cookies from a local cmux browser surface.
    Cmux(ImportCmuxSessionCommand),
    /// Import scoped cookies/storage from an explicitly selected browser profile.
    Browser(ImportBrowserSessionCommand),
    /// Import scoped cookies/storage from a Chrome profile snapshot.
    Chrome(ImportChromeSessionCommand),
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ImportCmuxSessionCommand {
    /// cmux browser surface to read cookies from.
    #[arg(long)]
    pub surface: String,

    /// Local session name to create.
    #[arg(long)]
    pub name: String,

    /// Explicit allowed cookie domain to import. Repeat for each trusted domain.
    #[arg(long, required = true)]
    pub allow_domain: Vec<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ImportBrowserSessionCommand {
    /// Browser family to import from. Chrome is the first supported import source.
    #[arg(long, value_enum, default_value_t = BrowserChoice::Chrome)]
    pub browser: BrowserChoice,

    /// Browser profile name or path for the selected browser.
    #[arg(long = "browser-profile")]
    pub browser_profile: Option<String>,

    /// Explicit filesystem path to a browser profile/user-data directory.
    #[arg(long = "profile-path")]
    pub profile_path: Option<PathBuf>,

    /// Local session name to create.
    #[arg(long)]
    pub name: String,

    /// Explicit allowed cookie/storage domain to import. Repeat for each trusted domain.
    #[arg(long, required = true)]
    pub allow_domain: Vec<String>,
}

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ImportChromeSessionCommand {
    /// Chrome profile name or path to snapshot through local Chrome/CDP.
    #[arg(long = "chrome-profile")]
    pub chrome_profile: String,

    /// Local session name to create.
    #[arg(long)]
    pub name: String,

    /// Explicit allowed cookie/storage domain to import. Repeat for each trusted domain.
    #[arg(long, required = true)]
    pub allow_domain: Vec<String>,
}
