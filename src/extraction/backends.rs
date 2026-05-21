use crate::aget_extractor::AgetExtractor;
use crate::error::AgetError;

use super::command::run_command_extractor_backend;
use super::fallback_command::run_agent_browser_fallback;
use super::{
    BrowserFallbackBackend, BrowserFallbackRequest, BrowserFallbackResult, ExtractorBackend,
    ExtractorBackendResult, ExtractorRequest,
};

const EXTRACTOR: &str = "crawl4ai";

#[derive(Debug, Clone)]
pub struct CommandBrowserFallbackBackend;

impl BrowserFallbackBackend for CommandBrowserFallbackBackend {
    fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError> {
        run_agent_browser_fallback(
            request.tmp_dir,
            request.url,
            request.state_path,
            request.options,
            request.timeout,
        )
    }
}

#[derive(Debug, Clone)]
pub struct CommandExtractorBackend {
    command: Option<String>,
}

impl CommandExtractorBackend {
    pub fn new(command: Option<String>) -> Self {
        Self { command }
    }
}

impl ExtractorBackend for CommandExtractorBackend {
    fn name(&self) -> &'static str {
        EXTRACTOR
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        run_command_extractor_backend(self.command.clone(), request)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AgetExtractorBackend {
    extractor: AgetExtractor,
}

impl AgetExtractorBackend {
    pub fn new(extractor: AgetExtractor) -> Self {
        Self { extractor }
    }
}

impl ExtractorBackend for AgetExtractorBackend {
    fn name(&self) -> &'static str {
        self.extractor.name()
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        self.extractor.extract(request)
    }
}
