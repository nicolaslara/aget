use crate::aget_extractor::AgetExtractor;
use crate::error::AgetError;

use super::{ExtractorBackend, ExtractorBackendResult, ExtractorRequest};

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
