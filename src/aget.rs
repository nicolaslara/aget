use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::aget_browser::AgetBrowser;
use crate::cli::{ExtractorOption, OutputFormat};
use crate::error::{AgetError, ErrorCode};
use crate::extraction::{
    AgetExtractorBackend, BrowserFallbackBackend, BrowserFallbackRequest, BrowserFallbackResult,
    CommandBrowserFallbackBackend, CommandExtractorBackend, ExtractionSessionStore,
    ExtractorBackend, GetOptions, GetSuccess,
};
use crate::session::{
    cancel_login_session as cancel_login_flow, complete_login_session, compose_session,
    finish_login_session as finish_login_flow, import_chrome_session as import_chrome_state,
    import_cmux_session as import_cmux_state, merge_login_session,
    start_login_session as start_login_flow, ChromeImportOptions, CmuxImportOptions,
    LoginCancelOptions, LoginCancelResult, LoginCompleteOptions, LoginFinishOptions,
    LoginFinishResult, LoginStartOptions, LoginStartResult, Session, SessionStore,
};

#[derive(Debug, Clone)]
pub struct AuthorizeSessionOptions {
    pub name: String,
    pub url: String,
    pub chrome_profile: String,
    pub allow_domains: Vec<String>,
    pub must_contain: Vec<String>,
    pub must_not_contain: Vec<String>,
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuthorizeSessionResult {
    pub state: AuthorizationState,
    pub name: String,
    pub source: &'static str,
    pub allowed_domains: Vec<String>,
    pub baseline: GetSuccess,
    pub verification: GetSuccess,
    pub predicates: Vec<AuthorizationPredicateResult>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationState {
    Verified,
    VerificationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AuthorizationPredicateResult {
    pub kind: &'static str,
    pub value: String,
    pub matched: bool,
}

pub type Aget = AgetWith<
    DefaultExtractorBackend,
    FilesystemSessionStoreBackend,
    DefaultBrowserAutomationBackend,
>;

#[derive(Clone)]
pub struct AgetWith<E, S, B> {
    // Session storage owns persisted local auth state. It is separate from browser
    // automation so future encrypted or test stores can reuse the same `Aget` flow.
    session_store: S,
    // Browser automation covers login/profile import flows. The default backend is
    // AgetBrowser CDP/Chrome automation; the command adapter remains compatibility-only.
    browser_backend: B,
    // Pluggable URL extraction capability. The default backend is AgetExtractor
    // extraction; command transport remains an explicit compatibility path.
    extractor_backend: E,
    timeout: Option<Duration>,
}

impl Aget {
    pub fn new(home: impl Into<PathBuf>) -> Self {
        Self {
            session_store: FilesystemSessionStoreBackend::new(home),
            browser_backend: DefaultBrowserAutomationBackend::aget(),
            extractor_backend: DefaultExtractorBackend::aget(),
            timeout: None,
        }
    }

    pub fn from_env() -> io::Result<Self> {
        let store = SessionStore::from_env()?;
        Ok(Self {
            session_store: FilesystemSessionStoreBackend::new(store.home().to_path_buf()),
            browser_backend: DefaultBrowserAutomationBackend::from_env(),
            extractor_backend: DefaultExtractorBackend::from_env(),
            timeout: None,
        })
    }
}

impl<S, B> AgetWith<DefaultExtractorBackend, S, B> {
    pub fn with_backend_command(mut self, command: impl Into<String>) -> Self {
        self.extractor_backend = DefaultExtractorBackend::command(Some(command.into()));
        self
    }
}

impl<E, S, B> AgetWith<E, S, B> {
    pub fn with_extractor_backend<NextE>(self, backend: NextE) -> AgetWith<NextE, S, B> {
        AgetWith {
            session_store: self.session_store,
            browser_backend: self.browser_backend,
            extractor_backend: backend,
            timeout: self.timeout,
        }
    }

    pub fn with_session_store_backend<NextS>(self, backend: NextS) -> AgetWith<E, NextS, B> {
        AgetWith {
            session_store: backend,
            browser_backend: self.browser_backend,
            extractor_backend: self.extractor_backend,
            timeout: self.timeout,
        }
    }

    pub fn with_browser_automation_backend<NextB>(self, backend: NextB) -> AgetWith<E, S, NextB> {
        AgetWith {
            session_store: self.session_store,
            browser_backend: backend,
            extractor_backend: self.extractor_backend,
            timeout: self.timeout,
        }
    }
}

impl<E, S, B> AgetWith<E, S, B>
where
    E: ExtractorBackend + Clone,
    S: SessionStoreBackend + Clone,
    B: BrowserAutomationBackend + BrowserFallbackBackend + Clone,
{
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_timeout_opt(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn home(&self) -> &Path {
        self.session_store.home()
    }

    pub fn list_sessions(&self) -> Result<Vec<String>, AgetError> {
        self.session_store.list().map_err(io_aget_error)
    }

    pub fn load_session(&self, name: &str) -> Result<Session, AgetError> {
        self.session_store.load(name).map_err(io_aget_error)
    }

    pub fn delete_session(&self, name: &str) -> Result<bool, AgetError> {
        self.session_store.delete(name).map_err(io_aget_error)
    }

    pub fn import_cmux_session(
        &self,
        surface: impl Into<String>,
        name: impl Into<String>,
        domains: Vec<String>,
    ) -> Result<Session, AgetError> {
        let session = import_cmux_state(CmuxImportOptions {
            surface: surface.into(),
            name: name.into(),
            domains,
            tmp_dir: self.tmp_dir(),
        })?;
        self.session_store.save(&session).map_err(io_aget_error)?;
        Ok(session)
    }

    pub fn import_chrome_session(
        &self,
        profile: impl Into<String>,
        name: impl Into<String>,
        domains: Vec<String>,
    ) -> Result<Session, AgetError> {
        let session = self.browser_backend.import_chrome(ChromeImportOptions {
            profile: profile.into(),
            name: name.into(),
            domains,
            tmp_dir: self.tmp_dir(),
        })?;
        self.session_store.save(&session).map_err(io_aget_error)?;
        Ok(session)
    }

    pub fn authorize_chrome_session(
        &self,
        options: AuthorizeSessionOptions,
    ) -> Result<AuthorizeSessionResult, AgetError> {
        let baseline = self
            .get(options.url.clone())
            .content_format(OutputFormat::Markdown)
            .run()?;
        let session = self.import_chrome_session(
            options.chrome_profile,
            options.name,
            options.allow_domains.clone(),
        )?;
        let mut verification_request = self
            .get(options.url)
            .session(session.name.clone())
            .content_format(OutputFormat::Markdown);
        if let Some(output) = options.output {
            verification_request = verification_request.output(output);
        }
        let verification = verification_request.run()?;
        let predicates = evaluate_authorization_predicates(
            &verification.content,
            &options.must_contain,
            &options.must_not_contain,
        );
        let verified = predicates.iter().all(|predicate| predicate.matched);
        let mut warnings = verification.warnings.clone();
        if !verified {
            warnings.push(
                "verification fetch completed, but one or more caller-supplied predicates failed"
                    .to_string(),
            );
        }
        Ok(AuthorizeSessionResult {
            state: if verified {
                AuthorizationState::Verified
            } else {
                AuthorizationState::VerificationFailed
            },
            name: session.name,
            source: "chrome",
            allowed_domains: options.allow_domains,
            baseline,
            verification,
            predicates,
            warnings,
        })
    }

    pub fn compose_sessions(
        &self,
        name: impl Into<String>,
        source_names: Vec<String>,
    ) -> Result<Session, AgetError> {
        let name = name.into();
        validate_compose_target(&self.session_store, &name, &source_names)?;
        let source_sessions = source_names
            .iter()
            .map(|source| self.session_store.load(source).map_err(io_aget_error))
            .collect::<Result<Vec<_>, _>>()?;
        let session = compose_session(&name, &source_sessions)?;
        self.session_store.save(&session).map_err(io_aget_error)?;
        Ok(session)
    }

    pub fn start_login_session(
        &self,
        name: impl Into<String>,
        profile: Option<String>,
        url: impl Into<String>,
    ) -> Result<LoginStartResult, AgetError> {
        self.browser_backend.start_login(LoginStartOptions {
            name: name.into(),
            profile,
            url: url.into(),
            tmp_dir: self.tmp_dir(),
        })
    }

    pub fn finish_login_session(&self, name: impl Into<String>) -> Result<Session, AgetError> {
        let result = self.browser_backend.finish_login(LoginFinishOptions {
            name: name.into(),
            tmp_dir: self.tmp_dir(),
        })?;
        let session = match self.session_store.load(&result.session.name) {
            Ok(existing) => merge_login_session(existing, result.session),
            Err(error) if error.kind() == io::ErrorKind::NotFound => result.session,
            Err(error) => return Err(io_aget_error(error)),
        };
        self.session_store.save(&session).map_err(io_aget_error)?;
        complete_login_session(LoginCompleteOptions {
            pending: result.pending,
            tmp_dir: self.tmp_dir(),
        })?;
        Ok(session)
    }

    pub fn cancel_login_session(
        &self,
        name: impl Into<String>,
    ) -> Result<LoginCancelResult, AgetError> {
        self.browser_backend.cancel_login(LoginCancelOptions {
            name: name.into(),
            tmp_dir: self.tmp_dir(),
        })
    }

    pub fn get(&self, url: impl Into<String>) -> GetRequest<E, B, S> {
        GetRequest {
            extractor_backend: self.extractor_backend.clone(),
            browser_fallback_backend: self.browser_backend.clone(),
            session_store: self.session_store.clone(),
            options: GetOptions {
                url: url.into(),
                sessions: Vec::new(),
                output: None,
                home: Some(self.session_store.home().to_path_buf()),
                timeout: self.timeout,
                content_format: OutputFormat::Markdown,
                selector: None,
                exclude_selector: None,
                wait_for_selector: None,
                max_chars: None,
                backend_options: Vec::new(),
            },
        }
    }

    fn tmp_dir(&self) -> PathBuf {
        self.session_store.home().join("tmp")
    }
}

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
    fn aget() -> Self {
        Self::Aget(AgetExtractorBackend::default())
    }

    fn command(command: Option<String>) -> Self {
        Self::Command(CommandExtractorBackend::new(command))
    }

    fn from_env() -> Self {
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
    fn aget() -> Self {
        Self::Aget(AgetBrowserBackend::default())
    }

    fn from_env() -> Self {
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

pub trait SessionStoreBackend {
    // The default implementation is filesystem-backed and local-first, but `Aget`
    // only needs this persistence contract.
    fn home(&self) -> &Path;
    fn list(&self) -> io::Result<Vec<String>>;
    fn load(&self, name: &str) -> io::Result<Session>;
    fn save(&self, session: &Session) -> io::Result<()>;
    fn delete(&self, name: &str) -> io::Result<bool>;
    fn exists(&self, name: &str) -> io::Result<bool>;
}

#[derive(Clone)]
pub struct FilesystemSessionStoreBackend {
    home: PathBuf,
}

impl FilesystemSessionStoreBackend {
    pub fn new(home: impl Into<PathBuf>) -> Self {
        Self { home: home.into() }
    }

    fn store(&self) -> io::Result<SessionStore> {
        SessionStore::new(&self.home)
    }
}

impl SessionStoreBackend for FilesystemSessionStoreBackend {
    fn home(&self) -> &Path {
        &self.home
    }

    fn list(&self) -> io::Result<Vec<String>> {
        self.store()?.list()
    }

    fn load(&self, name: &str) -> io::Result<Session> {
        self.store()?.load(name)
    }

    fn save(&self, session: &Session) -> io::Result<()> {
        self.store()?.save(session)
    }

    fn delete(&self, name: &str) -> io::Result<bool> {
        self.store()?.delete(name)
    }

    fn exists(&self, name: &str) -> io::Result<bool> {
        self.store()?.exists(name)
    }
}

impl<T> ExtractionSessionStore for T
where
    T: SessionStoreBackend,
{
    fn home(&self) -> &Path {
        SessionStoreBackend::home(self)
    }

    fn load(&self, name: &str) -> io::Result<Session> {
        SessionStoreBackend::load(self, name)
    }
}

pub struct GetRequest<E, B, S> {
    extractor_backend: E,
    browser_fallback_backend: B,
    session_store: S,
    options: GetOptions,
}

impl<E, B, S> GetRequest<E, B, S>
where
    E: ExtractorBackend,
    B: BrowserFallbackBackend,
    S: ExtractionSessionStore,
{
    pub fn session(mut self, name: impl Into<String>) -> Self {
        self.options.sessions.push(name.into());
        self
    }

    pub fn output(mut self, path: impl Into<PathBuf>) -> Self {
        self.options.output = Some(path.into());
        self
    }

    pub fn content_format(mut self, format: OutputFormat) -> Self {
        self.options.content_format = format;
        self
    }

    pub fn selector(mut self, selector: impl Into<String>) -> Self {
        self.options.selector = Some(selector.into());
        self
    }

    pub fn exclude_selector(mut self, exclude_selector: impl Into<String>) -> Self {
        self.options.exclude_selector = Some(exclude_selector.into());
        self
    }

    pub fn wait_for_selector(mut self, wait_for: impl Into<String>) -> Self {
        self.options.wait_for_selector = Some(wait_for.into());
        self
    }

    pub fn max_chars(mut self, max_chars: usize) -> Self {
        self.options.max_chars = Some(max_chars);
        self
    }

    pub fn backend_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.backend_options.push(ExtractorOption {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    pub fn run(self) -> Result<GetSuccess, AgetError> {
        crate::extraction::get_url_with_session_store(
            self.options,
            &self.session_store,
            &self.extractor_backend,
            &self.browser_fallback_backend,
        )
    }
}

fn evaluate_authorization_predicates(
    content: &str,
    must_contain: &[String],
    must_not_contain: &[String],
) -> Vec<AuthorizationPredicateResult> {
    let mut predicates = must_contain
        .iter()
        .map(|value| AuthorizationPredicateResult {
            kind: "must_contain",
            value: value.clone(),
            matched: content.contains(value),
        })
        .collect::<Vec<_>>();
    predicates.extend(
        must_not_contain
            .iter()
            .map(|value| AuthorizationPredicateResult {
                kind: "must_not_contain",
                value: value.clone(),
                matched: !content.contains(value),
            }),
    );
    predicates
}

fn validate_compose_target(
    store: &impl SessionStoreBackend,
    target: &str,
    sources: &[String],
) -> Result<(), AgetError> {
    if sources.iter().any(|source| source == target) {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("compose target '{target}' must not match a source session"),
        });
    }
    if store.exists(target).map_err(io_aget_error)? {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("session '{target}' already exists"),
        });
    }
    Ok(())
}

fn io_aget_error(error: impl std::fmt::Display) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}
