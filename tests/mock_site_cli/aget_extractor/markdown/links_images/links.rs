use std::path::Path;

use crate::support::mock_site::MockSite;

use super::super::markdown_content;
use super::support::LinkImageUrls;

pub(super) fn assert_automatic_links_option(aget_home: &Path, site: &MockSite) {
    let markdown_disable_automatic_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("aget.use_automatic_links", "false")],
    );
    assert!(markdown_disable_automatic_links.contains(
        "Canonical [https://example.com/docs](https://example.com/docs \"Docs title\")."
    ));
    assert!(!markdown_disable_automatic_links.contains("Canonical <https://example.com/docs>."));
}

pub(super) fn assert_link_options(aget_home: &Path, site: &MockSite, urls: &LinkImageUrls) {
    let markdown_ignore_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("aget.ignore_links", "true")],
    );
    assert!(markdown_ignore_links.contains("## Linked Heading"));
    assert!(markdown_ignore_links.contains("Read the guide or email support."));
    assert!(markdown_ignore_links.contains("Icon ![Download \\[app\\]]("));
    assert!(!markdown_ignore_links.contains("[the guide]("));
    assert!(!markdown_ignore_links.contains("[Linked Heading]("));
    assert!(!markdown_ignore_links.contains("Guide \\\"title\\\""));
    assert!(!markdown_ignore_links.contains(&site.url("/download")));

    let markdown_ignore_anchors = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("aget.ignore_anchors", "true")],
    );
    assert!(markdown_ignore_anchors.contains("## Linked Heading"));
    assert!(markdown_ignore_anchors.contains("Read the guide or email support."));
    assert!(markdown_ignore_anchors.contains("Icon ![Download \\[app\\]]("));
    assert!(!markdown_ignore_anchors.contains("[the guide]("));
    assert!(!markdown_ignore_anchors.contains("[Linked Heading]("));
    assert!(!markdown_ignore_anchors.contains("Guide \\\"title\\\""));
    assert!(!markdown_ignore_anchors.contains(&site.url("/download")));

    let markdown_include_mailto_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("aget.ignore_mailto_links", "false")],
    );
    assert!(markdown_include_mailto_links.contains("Read [the guide]("));
    assert!(markdown_include_mailto_links.contains("[email support](mailto:help@example.com)."));

    let markdown_protect_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("aget.protect_links", "true")],
    );
    assert!(markdown_protect_links.contains(&format!(
        "## [Linked Heading](<{}> \"Heading title\")",
        site.url("/linked-heading")
    )));
    assert!(markdown_protect_links.contains(&format!(
        "[the guide](<{}> \"Guide \\\"title\\\" \\[v1\\] \\(draft\\)\")",
        site.url("/guide")
    )));
    assert!(markdown_protect_links.contains(&format!(
        "[release notes](<{}>)",
        site.url("/release(2026)")
    )));
    assert!(markdown_protect_links.contains("<https://example.com/docs>"));
    assert!(markdown_protect_links.contains(&format!(
        "[![Download \\[app\\]]({})](<{}>)",
        urls.icon,
        site.url("/download")
    )));

    let markdown_skip_internal_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("aget.skip_internal_links", "true")],
    );
    assert!(markdown_skip_internal_links.contains("Jump within page."));
    assert!(!markdown_skip_internal_links.contains("/markdown-links#details"));
}
