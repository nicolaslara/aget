use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use aget::aget::{BrowserAutomationBackend, SessionStoreBackend};
use aget::extraction::{
    BrowserFallbackBackend, BrowserFallbackRequest, BrowserFallbackResult, ExtractorBackend,
    ExtractorBackendResult, ExtractorRequest,
};
use aget::session::login::PendingLogin;
use aget::{
    AgetError, ChromeImportOptions, ErrorCode, LoginCancelOptions, LoginCancelResult,
    LoginFinishOptions, LoginFinishResult, LoginStartOptions, LoginStartResult, Session,
    SessionCookie, SessionSource,
};

#[derive(Clone)]
pub(crate) struct MemorySessionStore {
    home: PathBuf,
    sessions: Rc<RefCell<BTreeMap<String, Session>>>,
}

impl MemorySessionStore {
    pub(crate) fn new(home: impl Into<PathBuf>) -> Self {
        Self {
            home: home.into(),
            sessions: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    pub(crate) fn load(&self, name: &str) -> io::Result<Session> {
        self.sessions
            .borrow()
            .get(name)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "session not found"))
    }

    pub(crate) fn save(&self, session: &Session) -> io::Result<()> {
        self.sessions
            .borrow_mut()
            .insert(session.name.clone(), session.clone());
        Ok(())
    }
}

impl SessionStoreBackend for MemorySessionStore {
    fn home(&self) -> &Path {
        &self.home
    }

    fn list(&self) -> io::Result<Vec<String>> {
        Ok(self.sessions.borrow().keys().cloned().collect())
    }

    fn load(&self, name: &str) -> io::Result<Session> {
        self.load(name)
    }

    fn save(&self, session: &Session) -> io::Result<()> {
        self.save(session)
    }

    fn delete(&self, name: &str) -> io::Result<bool> {
        Ok(self.sessions.borrow_mut().remove(name).is_some())
    }

    fn exists(&self, name: &str) -> io::Result<bool> {
        Ok(self.sessions.borrow().contains_key(name))
    }
}

#[derive(Clone)]
pub(crate) struct InspectingExtractor;

impl ExtractorBackend for InspectingExtractor {
    fn name(&self) -> &'static str {
        "inspect-extractor"
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        let state: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(request.state_path).unwrap()).unwrap();
        assert_eq!(state["cookies"][0]["value"], "memory-secret");
        assert_eq!(request.state.cookies[0].value, "memory-secret");
        assert_eq!(request.options.selector.as_deref(), Some("main"));
        assert!(request.content_path.starts_with(
            request
                .state_path
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("runs")
        ));
        assert!(request
            .metadata_path
            .starts_with(request.content_path.parent().unwrap()));

        Ok(ExtractorBackendResult {
            ok: true,
            final_url: Some(format!("{}/final", request.url)),
            content: Some("typed extractor content".to_string()),
            warnings: vec!["custom extractor".to_string()],
            error: None,
        })
    }
}

#[derive(Clone)]
pub(crate) struct FailingExtractor;

impl ExtractorBackend for FailingExtractor {
    fn name(&self) -> &'static str {
        "failing-extractor"
    }

    fn extract(&self, _request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        Err(AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: "primary extractor failed".to_string(),
        })
    }
}

#[derive(Clone)]
pub(crate) struct AuthorizationExtractor {
    baseline_content: String,
    verification_content: String,
    calls: Rc<RefCell<Vec<Vec<String>>>>,
}

impl AuthorizationExtractor {
    pub(crate) fn new(baseline_content: &str, verification_content: &str) -> Self {
        Self {
            baseline_content: baseline_content.to_string(),
            verification_content: verification_content.to_string(),
            calls: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub(crate) fn call_count(&self) -> usize {
        self.calls.borrow().len()
    }
}

impl ExtractorBackend for AuthorizationExtractor {
    fn name(&self) -> &'static str {
        "authorization-extractor"
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        self.calls
            .borrow_mut()
            .push(request.options.sessions.clone());
        let content = if request.options.sessions.is_empty() {
            &self.baseline_content
        } else {
            assert_eq!(request.state.cookies[0].value, "chrome-secret");
            &self.verification_content
        };

        Ok(ExtractorBackendResult {
            ok: true,
            final_url: Some(request.url.to_string()),
            content: Some(content.clone()),
            warnings: Vec::new(),
            error: None,
        })
    }
}

#[derive(Clone, Default)]
pub(crate) struct TestBrowserBackend {
    pub(crate) fallback_content: Option<String>,
    pub(crate) login_session: Option<Session>,
    pub(crate) import_error: Option<(ErrorCode, String)>,
}

impl BrowserFallbackBackend for TestBrowserBackend {
    fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError> {
        let state = fs::read_to_string(request.state_path).unwrap();
        assert!(state.contains("fallback-secret"));
        assert_eq!(request.state.cookies[0].value, "fallback-secret");
        assert!(request.tmp_dir.ends_with("tmp"));

        Ok(BrowserFallbackResult {
            final_url: request.url.to_string(),
            content: self
                .fallback_content
                .clone()
                .expect("fallback content should be configured"),
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
        Ok(LoginStartResult {
            pending: pending_login(
                options.name,
                options.profile.unwrap_or_default(),
                options.url,
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
            ),
        })
    }

    fn cancel_login(&self, options: LoginCancelOptions) -> Result<LoginCancelResult, AgetError> {
        Ok(LoginCancelResult {
            pending: pending_login(
                options.name,
                "test-profile".to_string(),
                "https://example.com/login".to_string(),
            ),
        })
    }
}

pub(crate) fn cookie_session(name: &str, domain: &str, value: &str) -> Session {
    let mut session = Session::new(name);
    session.source = SessionSource::Manual;
    session.allowed_cookie_domains = vec![domain.to_string()];
    session.cookies = vec![SessionCookie {
        name: "sid".to_string(),
        value: value.to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    }];
    session
}

pub(crate) fn pending_login(name: String, profile: String, url: String) -> PendingLogin {
    PendingLogin {
        agent_session: format!("aget-login-{name}"),
        allowed_domains: vec!["example.com".to_string()],
        browser_pid: None,
        name,
        profile,
        url,
    }
}

fn test_backend_error() -> AgetError {
    AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: "test backend not configured".to_string(),
    }
}
