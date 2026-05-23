use std::path::Path;

use crate::support::mock_site::MockSite;

use super::super::markdown_content;
use super::support::LinkImageUrls;

pub(super) fn assert_default_links_and_images(
    aget_home: &Path,
    site: &MockSite,
    urls: &LinkImageUrls,
) {
    let markdown_links = markdown_content(aget_home, site, "/markdown-links", &[]);
    assert_eq!(
        markdown_links,
        format!(
            "# Link Defaults\n\n## [Linked Heading]({} \"Heading title\")\n\nRead [the guide]({} \"Guide \\\"title\\\" \\[v1\\] \\(draft\\)\") or email support.\n\nCanonical <https://example.com/docs>.\n\nJump [within page]({}).\n\nLink label [API v1]({}) and standalone `inline_code`.\n\nEmpty []({}) marker.\n\nAsset [release notes]({}) and ![A \\[diagram\\] \\(v1\\)]({}).\n\nIcon [![Download \\[app\\]]({})]({}).\n\nMissing ![]({}) and empty ![]({}).",
            site.url("/linked-heading"),
            site.url("/guide"),
            site.url("/markdown-links#details"),
            site.url("/api"),
            site.url("/empty"),
            urls.release,
            urls.diagram,
            urls.icon,
            site.url("/download"),
            site.url("/missing-alt.png"),
            site.url("/empty-alt.png")
        )
    );
}
