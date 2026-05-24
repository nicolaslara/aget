use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use aget::aget::BrowserAutomationBackend;
use aget::extraction::{BrowserFallbackBackend, BrowserFallbackRequest, BrowserFallbackResult};
use aget::{
    AgetError, ChromeImportOptions, ErrorCode, LoginCancelOptions, LoginCancelResult,
    LoginFinishOptions, LoginFinishResult, LoginStartOptions, LoginStartResult, Session,
};

use super::session::pending_login;

#[derive(Clone, Default)]
pub(crate) struct TestBrowserBackend {
    pub(crate) fallback_content: Option<String>,
    pub(crate) login_session: Option<Session>,
    pub(crate) import_error: Option<(ErrorCode, String)>,
    pub(crate) login_starts: Rc<RefCell<Vec<LoginStartOptions>>>,
}

impl BrowserFallbackBackend for TestBrowserBackend {
    fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError> {
        let Some(content) = self.fallback_content.clone() else {
            return Err(test_backend_error());
        };
        let state = fs::read_to_string(request.state_path).unwrap();
        assert!(state.contains("fallback-secret"));
        assert_eq!(request.state.cookies[0].value, "fallback-secret");
        assert!(request.tmp_dir.ends_with("tmp"));

        Ok(BrowserFallbackResult {
            final_url: request.url.to_string(),
            content,
            page_metadata: Default::default(),
            warnings: vec!["custom browser fallback".to_string()],
            extractor: "test-browser-fallback".to_string(),
        })
    }
}

impl BrowserAutomationBackend for TestBrowserBackend {
    fn import_chrome(&self, _options: ChromeImportOptions) -> Result<Session, AgetError> {
        if let Some((code, message)) = &self.import_error {
            return Err(AgetError::Stable {
                code: *code,
                message: message.clone(),
            });
        }
        self.login_session.clone().ok_or_else(test_backend_error)
    }

    fn start_login(&self, options: LoginStartOptions) -> Result<LoginStartResult, AgetError> {
        self.login_starts.borrow_mut().push(options.clone());
        let injected_sessions = options
            .injected_sessions
            .iter()
            .map(|session| session.name.clone())
            .collect();
        Ok(LoginStartResult {
            pending: pending_login(
                options.name,
                options.profile.unwrap_or_default(),
                options.url,
                injected_sessions,
            ),
        })
    }

    fn finish_login(&self, options: LoginFinishOptions) -> Result<LoginFinishResult, AgetError> {
        Ok(LoginFinishResult {
            session: self.login_session.clone().ok_or_else(test_backend_error)?,
            pending: pending_login(
                options.name,
                "test-profile".to_string(),
                "https://example.com/login".to_string(),
                Vec::new(),
            ),
        })
    }

    fn cancel_login(&self, options: LoginCancelOptions) -> Result<LoginCancelResult, AgetError> {
        Ok(LoginCancelResult {
            pending: pending_login(
                options.name,
                "test-profile".to_string(),
                "https://example.com/login".to_string(),
                Vec::new(),
            ),
        })
    }
}

fn test_backend_error() -> AgetError {
    AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: "test backend not configured".to_string(),
    }
}
