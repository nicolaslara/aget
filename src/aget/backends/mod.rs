mod browser;
mod command_browser;
mod current_tab;
mod extractor;
mod owned_browser;

pub use self::browser::{BrowserAutomationBackend, DefaultBrowserAutomationBackend};
pub use self::command_browser::CommandBrowserAutomationBackend;
pub use self::current_tab::{
    BrowserCurrentTabBackend, BrowserCurrentTabRequest, BrowserCurrentTabResult,
};
pub use self::extractor::DefaultExtractorBackend;
pub use self::owned_browser::AgetBrowserBackend;
