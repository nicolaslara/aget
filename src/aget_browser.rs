mod current_tab;
#[cfg(test)]
mod tests;

pub(crate) use self::current_tab::CurrentTabRequest;

use crate::error::AgetError;
use crate::extraction::{BrowserFallbackRequest, BrowserFallbackResult};
use crate::session::{
    cancel_owned_login_session, finish_owned_login_session, import_owned_chrome_session,
    start_owned_login_session, ChromeImportOptions, LoginCancelOptions, LoginCancelResult,
    LoginFinishOptions, LoginFinishResult, LoginStartOptions, LoginStartResult, Session,
};

/// Local Chrome/CDP engine for browser-backed session capture and extraction.
#[derive(Clone, Debug, Default)]
pub struct AgetBrowser;

impl AgetBrowser {
    pub(crate) fn import_chrome_session(
        &self,
        options: ChromeImportOptions,
    ) -> Result<Session, AgetError> {
        import_owned_chrome_session(options)
    }

    pub(crate) fn start_login_session(
        &self,
        options: LoginStartOptions,
    ) -> Result<LoginStartResult, AgetError> {
        start_owned_login_session(options)
    }

    pub(crate) fn finish_login_session(
        &self,
        options: LoginFinishOptions,
    ) -> Result<LoginFinishResult, AgetError> {
        finish_owned_login_session(options)
    }

    pub(crate) fn cancel_login_session(
        &self,
        options: LoginCancelOptions,
    ) -> Result<LoginCancelResult, AgetError> {
        cancel_owned_login_session(options)
    }

    pub(crate) fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError> {
        crate::extraction::run_owned_browser_fallback(request)
    }
}
