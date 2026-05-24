use std::path::Path;

use crate::support::mock_site::MockSite;

use super::super::markdown_content;

pub(super) fn assert_google_doc_and_preserve_tags(aget_home: &Path, site: &MockSite) {
    let markdown_google_doc_default =
        markdown_content(aget_home, site, "/markdown-google-doc", &[]);
    assert!(markdown_google_doc_default.contains("Bold Italic Code Gone"));
    assert!(!markdown_google_doc_default.contains("**Bold**"));
    assert!(!markdown_google_doc_default.contains("_Italic_"));
    assert!(!markdown_google_doc_default.contains("`Code`"));

    let markdown_google_doc = markdown_content(
        aget_home,
        site,
        "/markdown-google-doc",
        &[("aget.google_doc", "true")],
    );
    assert!(markdown_google_doc.contains("**Bold** _Italic_ `Code` Gone"));

    let markdown_google_doc_hide_strike = markdown_content(
        aget_home,
        site,
        "/markdown-google-doc",
        &[
            ("aget.google_doc", "true"),
            ("aget.hide_strikethrough", "true"),
        ],
    );
    assert!(markdown_google_doc_hide_strike.contains("**Bold** _Italic_ `Code`"));
    assert!(!markdown_google_doc_hide_strike.contains("Gone"));

    let markdown_preserve_default =
        markdown_content(aget_home, site, "/markdown-preserve-tags", &[]);
    assert_eq!(
        markdown_preserve_default,
        "# Preserve Tags\n\nBefore **Raw** HTML after.\n\nSecond _Equation_ done."
    );

    let markdown_preserve_tags = markdown_content(
        aget_home,
        site,
        "/markdown-preserve-tags",
        &[("aget.preserve_tags", "custom-card,math-box")],
    );
    assert_eq!(
        markdown_preserve_tags,
        concat!(
            "# Preserve Tags\n\n",
            "Before\n\n",
            "<custom-card><strong>Raw</strong><span> HTML</span></custom-card>\n\n",
            "after.\n\n",
            "Second\n\n",
            "<math-box><em>Equation</em></math-box>\n\n",
            "done."
        )
    );
}
