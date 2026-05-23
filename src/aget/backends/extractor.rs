use crate::error::AgetError;
use crate::extraction::{
    AgetExtractorBackend, CommandExtractorBackend, ExtractorBackend, ExtractorBackendResult,
    ExtractorRequest,
};

#[derive(Clone)]
pub enum DefaultExtractorBackend {
    Aget(AgetExtractorBackend),
    Command(CommandExtractorBackend),
}

impl DefaultExtractorBackend {
    pub(in crate::aget) fn aget() -> Self {
        Self::Aget(AgetExtractorBackend::default())
    }

    pub(in crate::aget) fn command(command: Option<String>) -> Self {
        Self::Command(CommandExtractorBackend::new(command))
    }

    pub(in crate::aget) fn from_env() -> Self {
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

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        match self {
            Self::Aget(backend) => backend.extract(request),
            Self::Command(backend) => backend.extract(request),
        }
    }
}
