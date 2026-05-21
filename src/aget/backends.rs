use crate::aget_browser::AgetBrowser;
use crate::error::AgetError;
use crate::extraction::{
    AgetExtractorBackend, BrowserFallbackBackend, BrowserFallbackRequest, BrowserFallbackResult,
    CommandBrowserFallbackBackend, CommandExtractorBackend, ExtractorBackend,
};
use crate::session::{
    cancel_login_session as cancel_login_flow, finish_login_session as finish_login_flow,
    import_chrome_session as import_chrome_state, start_login_session as start_login_flow,
    ChromeImportOptions, LoginCancelOptions, LoginCancelResult, LoginFinishOptions,
    LoginFinishResult, LoginStartOptions, LoginStartResult, Session,
};

pub trait BrowserAutomationBackend {
    // Browser-backed session capture is modeled as a capability instead of as
    // command argv so callers do not couple to any particular backend transport.
    fn import_chrome(&self, options: ChromeImportOptions) -> Result<Session, AgetError>;
    fn start_login(&self, options: LoginStartOptions) -> Result<LoginStartResult, AgetError>;
    fn finish_login(&self, options: LoginFinishOptions) -> Result<LoginFinishResult, AgetError>;
    fn cancel_login(&self, options: LoginCancelOptions) -> Result<LoginCancelResult, AgetError>;
}

#[derive(Clone)]
pub enum DefaultExtractorBackend {
    Aget(AgetExtractorBackend),
    Command(CommandExtractorBackend),
}

impl DefaultExtractorBackend {
    pub(super) fn aget() -> Self {
        Self::Aget(AgetExtractorBackend::default())
    }

    pub(super) fn command(command: Option<String>) -> Self {
        Self::Command(CommandExtractorBackend::new(command))
    }

    pub(super) fn from_env() -> Self {
        match std::env::var("AGET_CRAWL4AI_COMMAND") {
            Ok(command) => Self::command(Some(command)),
            Err(_) => Self::aget(),
        }
    }
}

impl ExtractorBackend for DefaultExtractorBackend {
    fn name(&self) -> &'static str {
        match self {
            Self::Aget(backend) => backend.name(),
            Self::Command(backend) => backend.name(),
        }
    }

    fn extract(
        &self,
        request: crate::extraction::ExtractorRequest<'_>,
    ) -> Result<crate::extraction::ExtractorBackendResult, AgetError> {
        match self {
            Self::Aget(backend) => backend.extract(request),
            Self::Command(backend) => backend.extract(request),
        }
    }
}

#[derive(Clone)]
pub enum DefaultBrowserAutomationBackend {
    Aget(AgetBrowserBackend),
    Command(CommandBrowserAutomationBackend),
}

impl DefaultBrowserAutomationBackend {
    pub(super) fn aget() -> Self {
        Self::Aget(AgetBrowserBackend::default())
    }

    pub(super) fn from_env() -> Self {
        if std::env::var("AGET_AGENT_BROWSER_COMMAND").is_ok() {
            Self::Command(CommandBrowserAutomationBackend)
        } else {
            Self::aget()
        }
    }
}

impl BrowserAutomationBackend for DefaultBrowserAutomationBackend {
    fn import_chrome(&self, options: ChromeImportOptions) -> Result<Session, AgetError> {
        match self {
            Self::Aget(backend) => backend.import_chrome(options),
            Self::Command(backend) => backend.import_chrome(options),
        }
    }

    fn start_login(&self, options: LoginStartOptions) -> Result<LoginStartResult, AgetError> {
        match self {
            Self::Aget(backend) => backend.start_login(options),
            Self::Command(backend) => backend.start_login(options),
        }
    }

    fn finish_login(&self, options: LoginFinishOptions) -> Result<LoginFinishResult, AgetError> {
        match self {
            Self::Aget(backend) => backend.finish_login(options),
            Self::Command(backend) => backend.finish_login(options),
        }
    }

    fn cancel_login(&self, options: LoginCancelOptions) -> Result<LoginCancelResult, AgetError> {
        match self {
            Self::Aget(backend) => backend.cancel_login(options),
            Self::Command(backend) => backend.cancel_login(options),
        }
    }
}

impl BrowserFallbackBackend for DefaultBrowserAutomationBackend {
    fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError> {
        match self {
            Self::Aget(backend) => backend.extract_with_state(request),
            Self::Command(backend) => backend.extract_with_state(request),
        }
    }
}

#[derive(Clone)]
pub struct CommandBrowserAutomationBackend;

impl BrowserAutomationBackend for CommandBrowserAutomationBackend {
    fn import_chrome(&self, options: ChromeImportOptions) -> Result<Session, AgetError> {
        import_chrome_state(options)
    }

    fn start_login(&self, options: LoginStartOptions) -> Result<LoginStartResult, AgetError> {
        start_login_flow(options)
    }

    fn finish_login(&self, options: LoginFinishOptions) -> Result<LoginFinishResult, AgetError> {
        finish_login_flow(options)
    }

    fn cancel_login(&self, options: LoginCancelOptions) -> Result<LoginCancelResult, AgetError> {
        cancel_login_flow(options)
    }
}

impl BrowserFallbackBackend for CommandBrowserAutomationBackend {
    fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError> {
        CommandBrowserFallbackBackend.extract_with_state(request)
    }
}

#[derive(Clone, Debug, Default)]
pub struct AgetBrowserBackend {
    browser: AgetBrowser,
}

impl AgetBrowserBackend {
    pub fn new(browser: AgetBrowser) -> Self {
        Self { browser }
    }
}

impl BrowserAutomationBackend for AgetBrowserBackend {
    fn import_chrome(&self, options: ChromeImportOptions) -> Result<Session, AgetError> {
        self.browser.import_chrome_session(options)
    }

    fn start_login(&self, options: LoginStartOptions) -> Result<LoginStartResult, AgetError> {
        self.browser.start_login_session(options)
    }

    fn finish_login(&self, options: LoginFinishOptions) -> Result<LoginFinishResult, AgetError> {
        self.browser.finish_login_session(options)
    }

    fn cancel_login(&self, options: LoginCancelOptions) -> Result<LoginCancelResult, AgetError> {
        self.browser.cancel_login_session(options)
    }
}

impl BrowserFallbackBackend for AgetBrowserBackend {
    fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError> {
        self.browser.extract_with_state(request)
    }
}
