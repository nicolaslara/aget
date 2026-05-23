use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BrowserChoice {
    Chrome,
    Chromium,
    Brave,
    Edge,
    Arc,
    Firefox,
    Safari,
}

impl BrowserChoice {
    pub fn as_str(self) -> &'static str {
        match self {
            BrowserChoice::Chrome => "chrome",
            BrowserChoice::Chromium => "chromium",
            BrowserChoice::Brave => "brave",
            BrowserChoice::Edge => "edge",
            BrowserChoice::Arc => "arc",
            BrowserChoice::Firefox => "firefox",
            BrowserChoice::Safari => "safari",
        }
    }
}
