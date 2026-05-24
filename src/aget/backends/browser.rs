use crate::error::AgetError;
use crate::extraction::{BrowserFallbackBackend, BrowserFallbackRequest, BrowserFallbackResult};
use crate::session::{
    ChromeImportOptions, LoginCancelOptions, LoginCancelResult, LoginFinishOptions,
    LoginFinishResult, LoginStartOptions, LoginStartResult, Session,
};

use super::{
    AgetBrowserBackend, BrowserCurrentTabBackend, BrowserCurrentTabRequest, BrowserCurrentTabResult,
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
pub enum DefaultBrowserAutomationBackend {
    Aget(AgetBrowserBackend),
}

impl DefaultBrowserAutomationBackend {
    pub(in crate::aget) fn aget() -> Self {
        Self::Aget(AgetBrowserBackend::default())
    }

    pub(in crate::aget) fn from_env() -> Self {
        Self::aget()
    }
}

impl BrowserAutomationBackend for DefaultBrowserAutomationBackend {
    fn import_chrome(&self, options: ChromeImportOptions) -> Result<Session, AgetError> {
        match self {
            Self::Aget(backend) => backend.import_chrome(options),
        }
    }

    fn start_login(&self, options: LoginStartOptions) -> Result<LoginStartResult, AgetError> {
        match self {
            Self::Aget(backend) => backend.start_login(options),
        }
    }

    fn finish_login(&self, options: LoginFinishOptions) -> Result<LoginFinishResult, AgetError> {
        match self {
            Self::Aget(backend) => backend.finish_login(options),
        }
    }

    fn cancel_login(&self, options: LoginCancelOptions) -> Result<LoginCancelResult, AgetError> {
        match self {
            Self::Aget(backend) => backend.cancel_login(options),
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
        }
    }
}

impl BrowserCurrentTabBackend for DefaultBrowserAutomationBackend {
    fn render_current_tab(
        &self,
        request: BrowserCurrentTabRequest,
    ) -> Result<BrowserCurrentTabResult, AgetError> {
        match self {
            Self::Aget(backend) => backend.render_current_tab(request),
        }
    }
}
