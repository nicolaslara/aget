use std::path::PathBuf;

use crate::cli::{CachePolicy, ExtractorOption, OutputFormat};
use crate::error::AgetError;
use crate::extraction::{
    BrowserFallbackBackend, ExtractionSessionStore, ExtractorBackend, GetOptions, GetSuccess,
};

pub struct GetRequest<E, B, S> {
    pub(super) extractor_backend: E,
    pub(super) browser_fallback_backend: B,
    pub(super) session_store: S,
    pub(super) options: GetOptions,
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

    pub fn cache_policy(mut self, policy: CachePolicy) -> Self {
        self.options.cache_policy = policy;
        self
    }

    pub fn cache_ttl(mut self, ttl: std::time::Duration) -> Self {
        self.options.cache_ttl = ttl;
        self
    }

    pub fn capture_screenshot(mut self) -> Self {
        self.options.debug.screenshot = true;
        self
    }

    pub fn capture_trace(mut self) -> Self {
        self.options.debug.trace = true;
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
