use std::path::Path;

use crate::support::mock_site::MockSite;

use super::markdown_content;

pub(super) fn assert_links_and_images(aget_home: &Path, site: &MockSite) {
    let markdown_links = markdown_content(aget_home, site, "/markdown-links", &[]);
    let release_url = site
        .url("/release(2026)")
        .replace('(', "\\(")
        .replace(')', "\\)");
    let diagram_url = site
        .url("/assets/diagram(1).png")
        .replace('(', "\\(")
        .replace(')', "\\)");
    let icon_url = site
        .url("/icons/app(1).svg")
        .replace('(', "\\(")
        .replace(')', "\\)");
    assert_eq!(
        markdown_links,
        format!(
            "# Link Defaults\n\n## [Linked Heading]({} \"Heading title\")\n\nRead [the guide]({} \"Guide \\\"title\\\" \\[v1\\] \\(draft\\)\") or email support.\n\nCanonical <https://example.com/docs>.\n\nJump [within page]({}).\n\nLink label [API v1]({}) and standalone `inline_code`.\n\nEmpty []({}) marker.\n\nAsset [release notes]({}) and ![A \\[diagram\\] \\(v1\\)]({}).\n\nIcon [![Download \\[app\\]]({})]({}).\n\nMissing ![]({}) and empty ![]({}).",
            site.url("/linked-heading"),
            site.url("/guide"),
            site.url("/markdown-links#details"),
            site.url("/api"),
            site.url("/empty"),
            release_url,
            diagram_url,
            icon_url,
            site.url("/download"),
            site.url("/missing-alt.png"),
            site.url("/empty-alt.png")
        )
    );

    assert_automatic_links_option(aget_home, site);
    assert_image_options(aget_home, site, &release_url);
    assert_link_options(aget_home, site, &icon_url);
    assert_reference_link_option(aget_home, site);
    assert_paragraph_reference_link_option(aget_home, site);
}

fn assert_automatic_links_option(aget_home: &Path, site: &MockSite) {
    let markdown_disable_automatic_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("crawl4ai.use_automatic_links", "false")],
    );
    assert!(markdown_disable_automatic_links.contains(
        "Canonical [https://example.com/docs](https://example.com/docs \"Docs title\")."
    ));
    assert!(!markdown_disable_automatic_links.contains("Canonical <https://example.com/docs>."));
}

fn assert_image_options(aget_home: &Path, site: &MockSite, release_url: &str) {
    let markdown_ignore_images = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("crawl4ai.ignore_images", "true")],
    );
    assert!(markdown_ignore_images.contains("Asset [release notes]("));
    assert!(markdown_ignore_images.contains(&format!("Icon []({}).", site.url("/download"))));
    assert!(!markdown_ignore_images.contains("![A \\[diagram\\]"));
    assert!(!markdown_ignore_images.contains("/assets/diagram"));
    assert!(!markdown_ignore_images.contains("![Download \\[app\\]]"));

    let markdown_images_to_alt = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("crawl4ai.images_to_alt", "true")],
    );
    assert!(markdown_images_to_alt.contains("Canonical <https://example.com/docs>."));
    assert!(markdown_images_to_alt.contains(&format!(
        "Asset [release notes]({release_url}) and A \\[diagram\\] \\(v1\\)."
    )));
    assert!(markdown_images_to_alt.contains(&format!(
        "Icon [Download \\[app\\]]({}).",
        site.url("/download")
    )));
    assert!(!markdown_images_to_alt.contains("![A \\[diagram\\]"));
    assert!(!markdown_images_to_alt.contains("/assets/diagram"));
    assert!(!markdown_images_to_alt.contains("![Download \\[app\\]]"));

    let markdown_default_image_alt = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("crawl4ai.default_image_alt", "Missing alt")],
    );
    assert!(markdown_default_image_alt.contains(&format!(
        "Missing ![Missing alt]({}) and empty ![Missing alt]({}).",
        site.url("/missing-alt.png"),
        site.url("/empty-alt.png")
    )));

    let markdown_default_alt_to_alt = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[
            ("crawl4ai.default_image_alt", "Missing alt"),
            ("crawl4ai.images_to_alt", "true"),
        ],
    );
    assert!(markdown_default_alt_to_alt.contains("Missing Missing alt and empty Missing alt."));

    let markdown_images_as_html = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[
            ("crawl4ai.images_as_html", "true"),
            ("crawl4ai.images_to_alt", "true"),
        ],
    );
    assert!(markdown_images_as_html.contains("Asset [release notes]("));
    assert!(markdown_images_as_html.contains(
        "and <img src='/assets/diagram(1).png' width='640' height='360' alt='A [diagram] (v1)' />."
    ));
    assert!(markdown_images_as_html.contains(&format!(
        "Icon [<img src='/icons/app(1).svg' width='32' alt='Download [app]' />]({}).",
        site.url("/download")
    )));
    assert!(!markdown_images_as_html.contains("![A \\[diagram\\]"));
    assert!(!markdown_images_as_html.contains("A \\[diagram\\] \\(v1\\)."));

    let markdown_default_alt_as_html = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[
            ("crawl4ai.default_image_alt", "Missing alt"),
            ("crawl4ai.images_as_html", "true"),
        ],
    );
    assert!(markdown_default_alt_as_html.contains(
        "Missing <img src='/missing-alt.png' alt='Missing alt' /> and empty <img src='/empty-alt.png' alt='Missing alt' />."
    ));

    let markdown_images_with_size = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[
            ("crawl4ai.images_with_size", "true"),
            ("crawl4ai.images_to_alt", "true"),
        ],
    );
    assert!(markdown_images_with_size.contains(
        "and <img src='/assets/diagram(1).png' width='640' height='360' alt='A [diagram] (v1)' />."
    ));
    assert!(markdown_images_with_size.contains(&format!(
        "Icon [<img src='/icons/app(1).svg' width='32' alt='Download [app]' />]({}).",
        site.url("/download")
    )));
    assert!(!markdown_images_with_size.contains("![A \\[diagram\\]"));
    assert!(!markdown_images_with_size.contains("A \\[diagram\\] \\(v1\\)."));

    let markdown_images_with_size_keeps_unsized_images = markdown_content(
        aget_home,
        site,
        "/markdown-inline-blocks",
        &[("crawl4ai.images_with_size", "true")],
    );
    assert!(markdown_images_with_size_keeps_unsized_images
        .contains(&format!("![Figure alt]({})", site.url("/figure.png"))));
    assert!(!markdown_images_with_size_keeps_unsized_images.contains("<img src='/figure.png'"));
}

fn assert_link_options(aget_home: &Path, site: &MockSite, icon_url: &str) {
    let markdown_ignore_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("crawl4ai.ignore_links", "true")],
    );
    assert!(markdown_ignore_links.contains("## Linked Heading"));
    assert!(markdown_ignore_links.contains("Read the guide or email support."));
    assert!(markdown_ignore_links.contains("Icon ![Download \\[app\\]]("));
    assert!(!markdown_ignore_links.contains("[the guide]("));
    assert!(!markdown_ignore_links.contains("[Linked Heading]("));
    assert!(!markdown_ignore_links.contains("Guide \\\"title\\\""));
    assert!(!markdown_ignore_links.contains(&site.url("/download")));

    let markdown_include_mailto_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("crawl4ai.ignore_mailto_links", "false")],
    );
    assert!(markdown_include_mailto_links.contains("Read [the guide]("));
    assert!(markdown_include_mailto_links.contains("[email support](mailto:help@example.com)."));

    let markdown_protect_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("crawl4ai.protect_links", "true")],
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
        "[![Download \\[app\\]]({icon_url})](<{}>)",
        site.url("/download")
    )));

    let markdown_skip_internal_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("crawl4ai.skip_internal_links", "true")],
    );
    assert!(markdown_skip_internal_links.contains("Jump within page."));
    assert!(!markdown_skip_internal_links.contains("/markdown-links#details"));
}

fn assert_reference_link_option(aget_home: &Path, site: &MockSite) {
    let markdown_reference_links = markdown_content(
        aget_home,
        site,
        "/markdown-links",
        &[("crawl4ai.inline_links", "false")],
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
            ("crawl4ai.inline_links", "false"),
            ("crawl4ai.links_each_paragraph", "true"),
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
