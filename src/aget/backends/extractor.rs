use crate::error::AgetError;
use crate::extraction::{
    AgetExtractorBackend, ExtractorBackend, ExtractorBackendResult, ExtractorRequest,
};

#[derive(Clone)]
pub enum DefaultExtractorBackend {
    Aget(AgetExtractorBackend),
}

impl DefaultExtractorBackend {
    pub(in crate::aget) fn aget() -> Self {
        Self::Aget(AgetExtractorBackend::default())
    }

    pub(in crate::aget) fn from_env() -> Self {
        Self::aget()
    }
}

impl ExtractorBackend for DefaultExtractorBackend {
    fn name(&self) -> &'static str {
        match self {
            Self::Aget(backend) => backend.name(),
        }
    }

    fn extract(&self, request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        match self {
            Self::Aget(backend) => backend.extract(request),
        }
    }
}
