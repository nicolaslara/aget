use std::path::Path;

use crate::support::mock_site::MockSite;

use super::super::markdown_content;
use super::support::LinkImageUrls;

pub(super) fn assert_image_options(aget_home: &Path, site: &MockSite, urls: &LinkImageUrls) {
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
        "Asset [release notes]({}) and A \\[diagram\\] \\(v1\\).",
        urls.release
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
