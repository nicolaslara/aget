use crate::error::{AgetError, ErrorCode};
use crate::extraction::{
    BrowserFallbackBackend, BrowserFallbackRequest, BrowserFallbackResult,
    CommandBrowserFallbackBackend,
};
use crate::session::{
    cancel_login_session as cancel_login_flow, finish_login_session as finish_login_flow,
    import_chrome_session as import_chrome_state, start_login_session as start_login_flow,
    ChromeImportOptions, LoginCancelOptions, LoginCancelResult, LoginFinishOptions,
    LoginFinishResult, LoginStartOptions, LoginStartResult, Session,
};

use super::{
    BrowserAutomationBackend, BrowserCurrentTabBackend, BrowserCurrentTabRequest,
    BrowserCurrentTabResult,
};

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

impl BrowserCurrentTabBackend for CommandBrowserAutomationBackend {
    fn render_current_tab(
        &self,
        _request: BrowserCurrentTabRequest,
    ) -> Result<BrowserCurrentTabResult, AgetError> {
        Err(AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: "current-tab extraction is available only through the owned browser backend"
                .to_string(),
        })
    }
}
