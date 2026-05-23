#[path = "validation/browser.rs"]
mod browser;
#[path = "validation/document.rs"]
mod document;
#[path = "validation/markdown.rs"]
mod markdown;
#[path = "validation/support.rs"]
mod support;

use std::path::Path;

use crate::support::mock_site::MockSite;

pub(super) fn assert_option_validation(aget_home: &Path, site: &MockSite) {
    document::assert_document_option_validation(aget_home, site);
    markdown::assert_markdown_option_validation(aget_home, site);
    browser::assert_browser_option_validation(aget_home, site);
}
