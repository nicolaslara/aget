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
    Aget, AgetError, AuthorizationState, AuthorizeSessionOptions, ChromeImportOptions, ErrorCode,
    LoginCancelOptions, LoginCancelResult, LoginFinishOptions, LoginFinishResult,
    LoginStartOptions, LoginStartResult, OutputFormat, OwnedBrowserAutomationBackend, Session,
    SessionCookie, SessionSource,
};

#[test]
fn aget_with_static_backends_uses_custom_session_store_and_extractor() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    store
        .save(&cookie_session("auth", "example.com", "memory-secret"))
        .unwrap();

    let result = Aget::new(&home)
        .with_session_store_backend(store)
        .with_extractor_backend(InspectingExtractor)
        .with_browser_automation_backend(TestBrowserBackend::default())
        .get("https://example.com/private")
        .session("auth")
        .content_format(OutputFormat::Text)
        .selector("main")
        .run()
        .unwrap();

    assert_eq!(result.extractor, "inspect-extractor");
    assert_eq!(result.final_url, "https://example.com/private/final");
    assert_eq!(result.content, "typed extractor content");
    assert_eq!(result.sessions, vec!["auth"]);
    assert!(result.sensitive);
    assert_eq!(result.warnings, vec!["custom extractor"]);
}

#[test]
fn aget_with_static_backends_uses_custom_browser_fallback_after_extractor_failure() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    store
        .save(&cookie_session("auth", "example.com", "fallback-secret"))
        .unwrap();

    let result = Aget::new(&home)
        .with_session_store_backend(store)
        .with_extractor_backend(FailingExtractor)
        .with_browser_automation_backend(TestBrowserBackend {
            fallback_content: Some("browser fallback content".to_string()),
            ..TestBrowserBackend::default()
        })
        .get("https://example.com/private")
        .session("auth")
        .run()
        .unwrap();

    assert_eq!(result.extractor, "test-browser-fallback");
    assert_eq!(result.content, "browser fallback content");
    assert_eq!(result.warnings, vec!["custom browser fallback"]);
    assert!(result.sensitive);
}

#[test]
fn aget_with_static_browser_backend_saves_login_session_to_custom_store() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    let browser = TestBrowserBackend {
        login_session: Some(cookie_session("login", "example.com", "login-secret")),
        ..TestBrowserBackend::default()
    };

    let aget = Aget::new(&home)
        .with_session_store_backend(store.clone())
        .with_browser_automation_backend(browser);
    let session = aget.finish_login_session("login").unwrap();

    assert_eq!(session.name, "login");
    assert_eq!(
        store.load("login").unwrap().cookies[0].value,
        "login-secret"
    );
}

#[test]
fn aget_with_static_browser_backend_imports_chrome_session_to_custom_store() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    let browser = TestBrowserBackend {
        login_session: Some(cookie_session("chrome", "example.com", "chrome-secret")),
        ..TestBrowserBackend::default()
    };

    let aget = Aget::new(&home)
        .with_session_store_backend(store.clone())
        .with_browser_automation_backend(browser);
    let session = aget
        .import_chrome_session("Default", "chrome", vec!["example.com".to_string()])
        .unwrap();

    assert_eq!(session.name, "chrome");
    assert_eq!(
        store.load("chrome").unwrap().cookies[0].value,
        "chrome-secret"
    );
}

#[test]
fn authorize_chrome_session_fetches_baseline_imports_and_verifies() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    let extractor = AuthorizationExtractor::new("Please sign in", "Welcome private content");
    let browser = TestBrowserBackend {
        login_session: Some(cookie_session("news", "example.com", "chrome-secret")),
        ..TestBrowserBackend::default()
    };

    let result = Aget::new(&home)
        .with_session_store_backend(store.clone())
        .with_extractor_backend(extractor.clone())
        .with_browser_automation_backend(browser)
        .authorize_chrome_session(AuthorizeSessionOptions {
            name: "news".to_string(),
            url: "https://example.com/account".to_string(),
            chrome_profile: "Default".to_string(),
            allow_domains: vec!["example.com".to_string()],
            must_contain: vec!["Welcome".to_string()],
            must_not_contain: vec!["Please sign in".to_string()],
            output: None,
        })
        .unwrap();

    assert_eq!(result.state, AuthorizationState::Verified);
    assert_eq!(result.source, "chrome");
    assert_eq!(result.baseline.content, "Please sign in");
    assert!(result.baseline.sessions.is_empty());
    assert!(!result.baseline.sensitive);
    assert_eq!(result.verification.content, "Welcome private content");
    assert_eq!(result.verification.sessions, vec!["news"]);
    assert!(result.verification.sensitive);
    assert!(result.predicates.iter().all(|predicate| predicate.matched));
    assert_eq!(extractor.call_count(), 2);
    assert_eq!(
        store.load("news").unwrap().cookies[0].value,
        "chrome-secret"
    );
}

#[test]
fn authorize_chrome_session_reports_failed_verification_predicates() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    let extractor = AuthorizationExtractor::new("Please sign in", "Please sign in");
    let browser = TestBrowserBackend {
        login_session: Some(cookie_session("news", "example.com", "chrome-secret")),
        ..TestBrowserBackend::default()
    };

    let result = Aget::new(&home)
        .with_session_store_backend(store.clone())
        .with_extractor_backend(extractor)
        .with_browser_automation_backend(browser)
        .authorize_chrome_session(AuthorizeSessionOptions {
            name: "news".to_string(),
            url: "https://example.com/account".to_string(),
            chrome_profile: "Default".to_string(),
            allow_domains: vec!["example.com".to_string()],
            must_contain: vec!["Welcome".to_string()],
            must_not_contain: vec!["Please sign in".to_string()],
            output: None,
        })
        .unwrap();

    assert_eq!(result.state, AuthorizationState::VerificationFailed);
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.contains("predicates failed")));
    assert!(result.predicates.iter().any(|predicate| {
        predicate.kind == "must_contain" && predicate.value == "Welcome" && !predicate.matched
    }));
    assert!(store.load("news").is_ok());
}

#[test]
fn authorize_chrome_session_preserves_requires_user_action_after_baseline_fetch() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let store = MemorySessionStore::new(&home);
    let extractor = AuthorizationExtractor::new("Please sign in", "Welcome private content");
    let browser = TestBrowserBackend {
        import_error: Some((
            ErrorCode::RequiresUserAction,
            "quit this browser/profile before import".to_string(),
        )),
        ..TestBrowserBackend::default()
    };

    let error = Aget::new(&home)
        .with_session_store_backend(store.clone())
        .with_extractor_backend(extractor.clone())
        .with_browser_automation_backend(browser)
        .authorize_chrome_session(AuthorizeSessionOptions {
            name: "news".to_string(),
            url: "https://example.com/account".to_string(),
            chrome_profile: "Default".to_string(),
            allow_domains: vec!["example.com".to_string()],
            must_contain: vec!["Welcome".to_string()],
            must_not_contain: Vec::new(),
            output: None,
        })
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::RequiresUserAction);
    assert!(error.to_string().contains("quit this browser/profile"));
    assert_eq!(extractor.call_count(), 1);
    assert!(store.load("news").is_err());
}

#[test]
fn owned_browser_backend_does_not_save_failed_profile_path_import() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let missing_profile = temp.path().join("missing-profile");

    let error = Aget::new(&home)
        .with_browser_automation_backend(OwnedBrowserAutomationBackend)
        .import_chrome_session(
            missing_profile.to_string_lossy().to_string(),
            "chrome",
            vec!["example.com".to_string()],
        )
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::RequiresUserAction);
    assert!(error.to_string().contains("does not exist"));
    assert!(!home.join("sessions/chrome.json").exists());
}

#[test]
fn owned_browser_backend_cancels_pending_login_without_agent_browser() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let tmp_dir = home.join("tmp");
    let profile = tmp_dir.join("owned-login/aget-docs");
    fs::create_dir_all(&profile).unwrap();
    fs::create_dir_all(&tmp_dir).unwrap();
    fs::write(
        tmp_dir.join("login-docs.json"),
        serde_json::to_vec_pretty(&pending_login(
            "docs".to_string(),
            profile.to_string_lossy().into_owned(),
            "https://example.com/login".to_string(),
        ))
        .unwrap(),
    )
    .unwrap();

    let cancelled = Aget::new(&home)
        .with_browser_automation_backend(OwnedBrowserAutomationBackend)
        .cancel_login_session("docs")
        .unwrap();

    assert_eq!(cancelled.pending.name, "docs");
    assert!(!profile.exists());
    assert!(!tmp_dir.join("login-docs.json").exists());
}

#[test]
fn aget_with_static_browser_backend_starts_and_cancels_login_without_command_backend() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let aget = Aget::new(&home)
        .with_session_store_backend(MemorySessionStore::new(&home))
        .with_browser_automation_backend(TestBrowserBackend::default());

    let started = aget
        .start_login_session(
            "docs",
            Some("custom-profile".to_string()),
            "https://example.com/login",
        )
        .unwrap();
    assert_eq!(started.pending.name, "docs");
    assert_eq!(started.pending.profile, "custom-profile");
    assert_eq!(started.pending.url, "https://example.com/login");

    let cancelled = aget.cancel_login_session("docs").unwrap();
    assert_eq!(cancelled.pending.name, "docs");
    assert_eq!(cancelled.pending.profile, "test-profile");
}

#[derive(Clone)]
struct MemorySessionStore {
    home: PathBuf,
    sessions: Rc<RefCell<BTreeMap<String, Session>>>,
}

impl MemorySessionStore {
    fn new(home: impl Into<PathBuf>) -> Self {
        Self {
            home: home.into(),
            sessions: Rc::new(RefCell::new(BTreeMap::new())),
        }
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
        self.sessions
            .borrow()
            .get(name)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "session not found"))
    }

    fn save(&self, session: &Session) -> io::Result<()> {
        self.sessions
            .borrow_mut()
            .insert(session.name.clone(), session.clone());
        Ok(())
    }

    fn delete(&self, name: &str) -> io::Result<bool> {
        Ok(self.sessions.borrow_mut().remove(name).is_some())
    }

    fn exists(&self, name: &str) -> io::Result<bool> {
        Ok(self.sessions.borrow().contains_key(name))
    }
}

#[derive(Clone)]
struct InspectingExtractor;

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
struct FailingExtractor;

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
struct AuthorizationExtractor {
    baseline_content: String,
    verification_content: String,
    calls: Rc<RefCell<Vec<Vec<String>>>>,
}

impl AuthorizationExtractor {
    fn new(baseline_content: &str, verification_content: &str) -> Self {
        Self {
            baseline_content: baseline_content.to_string(),
            verification_content: verification_content.to_string(),
            calls: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn call_count(&self) -> usize {
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
struct TestBrowserBackend {
    fallback_content: Option<String>,
    login_session: Option<Session>,
    import_error: Option<(ErrorCode, String)>,
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

fn cookie_session(name: &str, domain: &str, value: &str) -> Session {
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

fn pending_login(name: String, profile: String, url: String) -> PendingLogin {
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
