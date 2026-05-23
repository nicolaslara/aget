use std::path::Path;

use aget::{Aget, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site::MockSite;

pub(super) fn html_cleanup(
    aget_home: &Path,
    site: &MockSite,
    backend_options: &[(&str, &str)],
) -> String {
    cleanup_content(aget_home, site, OutputFormat::Html, backend_options, None)
}

pub(super) fn cleanup_content(
    aget_home: &Path,
    site: &MockSite,
    format: OutputFormat,
    backend_options: &[(&str, &str)],
    selector: Option<&str>,
) -> String {
    let mut request = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(format);
    if let Some(selector) = selector {
        request = request.selector(selector);
    }
    for (key, value) in backend_options {
        request = request.backend_option(*key, *value);
    }
    request.run().unwrap().content
}
