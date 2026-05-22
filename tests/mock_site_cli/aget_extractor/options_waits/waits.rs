use std::path::Path;

use aget::{Aget, AgetExtractorBackend, ErrorCode, OutputFormat};

use crate::support::mock_site::MockSite;

pub(super) fn assert_redirects_and_waits(aget_home: &Path, site: &MockSite) {
    let redirect = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/redirect"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(redirect.final_url, site.url("/public"));

    let waited = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/wait-ready"))
        .content_format(OutputFormat::Text)
        .wait_for_selector("css:body main #ready")
        .run()
        .unwrap();
    assert_eq!(waited.content, "Ready Now");

    let js_wait = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/wait-ready"))
        .wait_for_selector("js:() => true")
        .run()
        .unwrap_err();
    assert_eq!(js_wait.code(), ErrorCode::ExtractionFailed);
}
