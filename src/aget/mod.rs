mod authorize;
mod backends;
mod get_request;
mod session_store;

use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use self::authorize::evaluate_authorization_predicates;
pub use self::authorize::{
    AuthorizationPredicateResult, AuthorizationState, AuthorizeSessionOptions,
    AuthorizeSessionResult,
};
pub use self::backends::{
    AgetBrowserBackend, BrowserAutomationBackend, BrowserCurrentTabBackend,
    BrowserCurrentTabRequest, CommandBrowserAutomationBackend, DefaultBrowserAutomationBackend,
    DefaultExtractorBackend,
};
pub use self::get_request::GetRequest;
pub use self::session_store::{FilesystemSessionStoreBackend, SessionStoreBackend};
use crate::cli::{ExtractorOption, OutputFormat};
use crate::error::{AgetError, ErrorCode};
use crate::extraction::{
    extract_owned_rendered_html, finish_direct_extraction, validate_owned_extraction_options,
    BrowserFallbackBackend, ExtractorBackend, GetOptions, GetSuccess,
};
use crate::session::{
    complete_login_session, compose_session, import_cmux_session as import_cmux_state,
    merge_login_session, ChromeImportOptions, CmuxImportOptions, LoginCancelOptions,
    LoginCancelResult, LoginCompleteOptions, LoginFinishOptions, LoginStartOptions,
    LoginStartResult, Session, SessionStore,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const CURRENT_TAB_EXTRACTOR: &str = "aget-owned-current-tab";

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

#[derive(Clone, Debug)]
pub struct CurrentTabOptions {
    pub port: u16,
    pub allow_private_content: bool,
    pub output: Option<PathBuf>,
    pub timeout: Option<Duration>,
    pub content_format: OutputFormat,
    pub selector: Option<String>,
    pub exclude_selector: Option<String>,
    pub wait_for_selector: Option<String>,
    pub max_chars: Option<usize>,
    pub backend_options: Vec<ExtractorOption>,
}

impl CurrentTabOptions {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            allow_private_content: false,
            output: None,
            timeout: None,
            content_format: OutputFormat::Markdown,
            selector: None,
            exclude_selector: None,
            wait_for_selector: None,
            max_chars: None,
            backend_options: Vec::new(),
        }
    }
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

impl<E, S, B> AgetWith<E, S, B>
where
    S: SessionStoreBackend + Clone,
    B: BrowserCurrentTabBackend + Clone,
{
    pub fn current_tab(&self, mut options: CurrentTabOptions) -> Result<GetSuccess, AgetError> {
        if !options.allow_private_content {
            return Err(AgetError::Stable {
                code: ErrorCode::UsageError,
                message: "current-tab reads the selected browser tab and may include authenticated/private content; pass explicit consent before running".to_string(),
            });
        }

        let started = std::time::Instant::now();
        let timeout = options.timeout.or(self.timeout).unwrap_or(DEFAULT_TIMEOUT);
        let get_options = GetOptions {
            url: "current-tab".to_string(),
            sessions: Vec::new(),
            output: options.output.take(),
            home: Some(self.session_store.home().to_path_buf()),
            timeout: Some(timeout),
            content_format: options.content_format,
            selector: options.selector.take(),
            exclude_selector: options.exclude_selector.take(),
            wait_for_selector: options.wait_for_selector.take(),
            max_chars: options.max_chars,
            backend_options: options.backend_options,
        };
        let owned_options = validate_owned_extraction_options(&get_options)?;
        let rendered = self
            .browser_backend
            .render_current_tab(BrowserCurrentTabRequest {
                port: options.port,
                wait_for_selector: get_options.wait_for_selector.clone(),
                wait_for_images: owned_options.wait_for_images,
                flatten_shadow_dom: owned_options.flatten_shadow_dom,
                settle_delay: owned_options.render_settle_delay,
                discovery_timeout: timeout,
                page_timeout: owned_options.page_timeout.unwrap_or(timeout),
                wait_for_timeout: owned_options.wait_for_timeout,
                timeout,
            })?;
        let mut extraction =
            extract_owned_rendered_html(rendered.final_url, rendered.html, &get_options)?;
        let mut warnings =
            vec!["current-tab content may include authenticated/private browser data".to_string()];
        warnings.push(format!(
            "current-tab used local CDP endpoint {}",
            rendered.cdp_ws_url
        ));
        warnings.extend(rendered.warnings);
        warnings.append(&mut extraction.warnings);

        finish_direct_extraction(
            get_options,
            extraction.final_url,
            extraction.content,
            warnings,
            CURRENT_TAB_EXTRACTOR,
            true,
            started,
        )
    }
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
