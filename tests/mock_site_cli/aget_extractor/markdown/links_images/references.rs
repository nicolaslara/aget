use std::path::Path;

use crate::support::mock_site::MockSite;

use super::super::markdown_content;

pub(super) fn assert_reference_link_options(aget_home: &Path, site: &MockSite) {
    assert_reference_link_option(aget_home, site);
    assert_paragraph_reference_link_option(aget_home, site);
}

fn assert_reference_link_option(aget_home: &Path, site: &MockSite) {
    let markdown_reference_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("aget.inline_links", "false")],
    );
    assert!(markdown_reference_links.contains("## [Linked Heading][1]"));
    assert!(markdown_reference_links.contains("Read [the guide][2] or email support."));
    assert!(markdown_reference_links.contains("Canonical <https://example.com/docs>."));
    assert!(markdown_reference_links.contains("Jump [within page][3]."));
    assert!(markdown_reference_links.contains("Link label [API v1][4]"));
    assert!(markdown_reference_links
        .contains("Asset [release notes][6] and ![A \\[diagram\\] \\(v1\\)][7]."));
    assert!(markdown_reference_links.contains("Icon [![Download \\[app\\]][8]][9]."));
    assert!(markdown_reference_links.contains(&format!(
        "   [1]: {} (Heading title)",
        site.url("/linked-heading")
    )));
    assert!(markdown_reference_links.contains(&format!(
        "   [2]: {} (Guide \\\"title\\\" \\[v1\\] \\(draft\\))",
        site.url("/guide")
    )));
    assert!(markdown_reference_links
        .contains(&format!("   [3]: {}", site.url("/markdown-links#details"))));
    assert!(markdown_reference_links.contains(&format!("   [9]: {}", site.url("/download"))));
    assert!(!markdown_reference_links.contains("[the guide]("));
}

fn assert_paragraph_reference_link_option(aget_home: &Path, site: &MockSite) {
    let markdown_reference_paragraphs = markdown_content(
        aget_home,
        site,
        "/markdown-reference-paragraphs",
        &[
            ("aget.inline_links", "false"),
            ("aget.links_each_paragraph", "true"),
        ],
    );
    assert_eq!(
        markdown_reference_paragraphs,
        format!(
            "# Reference Paragraphs\n\nFirst [alpha][1] and [beta][2].\n\n   [1]: {}\n   [2]: {}\n\nSecond [alpha again][3].\n\n   [3]: {}",
            site.url("/alpha"),
            site.url("/beta"),
            site.url("/alpha")
        )
    );
}
