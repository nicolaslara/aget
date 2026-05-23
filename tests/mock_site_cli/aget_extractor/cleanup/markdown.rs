use std::path::Path;

use aget::OutputFormat;

use crate::support::mock_site::MockSite;

use super::support::cleanup_content;

pub(super) fn assert_markdown_cleanup_output(aget_home: &Path, site: &MockSite) {
    let cleanup_markdown = cleanup_content(aget_home, site, OutputFormat::Markdown, &[], None);
    assert!(!cleanup_markdown.contains("data:image"));
    assert!(!cleanup_markdown.contains("![Inline image]"));
}
