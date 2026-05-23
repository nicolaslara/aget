use std::path::Path;

use crate::support::mock_site::MockSite;

use super::support::html_cleanup;

pub(super) fn assert_default_cleaned_html(aget_home: &Path, site: &MockSite) {
    let cleaned_html = html_cleanup(aget_home, site, &[]);
    assert!(cleaned_html.contains("<title>Cleanup Title</title>"));
    assert!(cleaned_html.contains("<h1>Cleanup Main</h1>"));
    assert!(cleaned_html.contains("id=\"cleanup-main\""));
    assert!(cleaned_html.contains("class=\"article\""));
    assert!(cleaned_html.contains("href=\"/kept\""));
    assert!(cleaned_html.contains("title=\"Kept title\""));
    assert!(cleaned_html.contains("id=\"same-domain-link\""));
    assert!(cleaned_html.contains("href=\"/same-domain\""));
    assert!(cleaned_html.contains("src=\"/diagram.png\""));
    assert!(cleaned_html.contains("alt=\"Diagram\""));
    assert!(cleaned_html.contains("width=\"640\""));
    assert!(cleaned_html.contains("height=\"480\""));
    assert!(cleaned_html.contains("id=\"inline-image\""));
    assert!(cleaned_html.contains("alt=\"Inline image\""));
    assert!(cleaned_html.contains("id=\"empty-anchor\""));
    assert!(cleaned_html.contains("href=\"/empty\""));
    assert!(cleaned_html.contains("id=\"external-link\""));
    assert!(cleaned_html.contains("href=\"https://external.example/out\""));
    assert!(cleaned_html.contains("id=\"mailto-link\""));
    assert!(cleaned_html.contains("href=\"mailto:help@example.com\""));
    assert!(cleaned_html.contains("id=\"social-link\""));
    assert!(cleaned_html.contains("href=\"https://www.linkedin.com/company/aget\""));
    assert!(cleaned_html.contains("id=\"custom-social-link\""));
    assert!(cleaned_html.contains("href=\"https://social.example/company/aget\""));
    assert!(cleaned_html.contains("id=\"remote-image\""));
    assert!(cleaned_html.contains("src=\"https://cdn.example/remote.png\""));
    assert!(cleaned_html.contains("id=\"social-image\""));
    assert!(cleaned_html.contains("src=\"https://www.linkedin.com/logo.png\""));
    assert!(cleaned_html.contains("id=\"empty-break\""));
    assert!(cleaned_html.contains("id=\"empty-row\""));
    assert!(cleaned_html.contains("id=\"empty-cell\""));
    assert!(cleaned_html.contains("id=\"code-space\""));
    assert!(!cleaned_html.contains("<meta"));
    assert!(!cleaned_html.contains("<link"));
    assert!(!cleaned_html.contains("<style"));
    assert!(!cleaned_html.contains("<script"));
    assert!(!cleaned_html.contains("<noscript"));
    assert!(!cleaned_html.contains("debug-secret"));
    assert!(!cleaned_html.contains("data-private"));
    assert!(!cleaned_html.contains("data-select"));
    assert!(!cleaned_html.contains("style="));
    assert!(!cleaned_html.contains("onclick="));
    assert!(!cleaned_html.contains("aria-label="));
    assert!(!cleaned_html.contains("rel=\"nofollow\""));
    assert!(!cleaned_html.contains("data:image/png;base64"));
    assert!(!cleaned_html.contains("QUJDRA"));
    assert!(!cleaned_html.contains("empty-wrapper"));
    assert!(!cleaned_html.contains("empty-span"));
}
