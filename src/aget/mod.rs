mod authorize;
mod backends;
mod current_tab;
mod get_request;
mod session_store;
mod sessions;

use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub use self::authorize::{
    AuthorizationPredicateResult, AuthorizationState, AuthorizeSessionOptions,
    AuthorizeSessionResult,
};
pub use self::backends::{
    AgetBrowserBackend, BrowserAutomationBackend, BrowserCurrentTabBackend,
    BrowserCurrentTabRequest, DefaultBrowserAutomationBackend, DefaultExtractorBackend,
};
pub use self::current_tab::CurrentTabOptions;
pub use self::get_request::GetRequest;
pub use self::session_store::{FilesystemSessionStoreBackend, SessionStoreBackend};
use crate::cli::OutputFormat;
use crate::error::{AgetError, ErrorCode};
use crate::extraction::{BrowserFallbackBackend, ExtractorBackend, GetOptions};
use crate::session::SessionStore;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

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
    // AgetBrowser CDP/Chrome automation.
    browser_backend: B,
    // Pluggable URL extraction capability. The default backend is AgetExtractor.
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

    pub(super) fn tmp_dir(&self) -> PathBuf {
        self.session_store.home().join("tmp")
    }
}

pub(super) fn io_aget_error(error: impl std::fmt::Display) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}
