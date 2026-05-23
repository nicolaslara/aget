use std::path::Path;

use aget::{Aget, AgetExtractorBackend, ErrorCode};

use crate::support::mock_site::MockSite;

pub(super) fn assert_invalid_option_contains(
    aget_home: &Path,
    site: &MockSite,
    key: &str,
    value: &str,
    expected_message: &str,
) {
    let error = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option(key, value)
        .run()
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::ExtractionFailed);
    assert!(
        error.to_string().contains(expected_message),
        "expected error for {key}={value:?} to contain {expected_message:?}, got {error}"
    );
}
