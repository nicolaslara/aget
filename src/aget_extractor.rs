use crate::error::AgetError;
use crate::extraction::{
    run_owned_extractor_backend, ExtractorBackendResult, ExtractorRequest, OWNED_EXTRACTOR,
};

/// Local extraction engine for the Crawl4AI-like URL-to-content behavior that
/// `aget` now owns. The methods currently delegate to the migrated extraction
/// slices while the engine boundary is introduced mechanically.
#[derive(Clone, Debug, Default)]
pub struct AgetExtractor;

impl AgetExtractor {
    pub(crate) fn name(&self) -> &'static str {
        OWNED_EXTRACTOR
    }

    pub(crate) fn extract(
        &self,
        request: ExtractorRequest<'_>,
    ) -> Result<ExtractorBackendResult, AgetError> {
        run_owned_extractor_backend(request)
    }
}
