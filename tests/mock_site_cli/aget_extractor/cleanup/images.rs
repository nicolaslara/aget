use std::path::Path;

use crate::support::mock_site::MockSite;

use super::support::html_cleanup;

pub(super) fn assert_image_cleanup_options(aget_home: &Path, site: &MockSite) {
    let no_images = html_cleanup(aget_home, site, &[("crawl4ai.exclude_all_images", "true")]);
    assert!(!no_images.contains("<img"));
    assert!(!no_images.contains("diagram.png"));
    assert!(!no_images.contains("Inline image"));

    let no_external_images = html_cleanup(
        aget_home,
        site,
        &[("crawl4ai.exclude_external_images", "true")],
    );
    assert!(no_external_images.contains("href=\"/kept\""));
    assert!(no_external_images.contains("href=\"https://external.example/out\""));
    assert!(no_external_images.contains("id=\"social-link\""));
    assert!(no_external_images.contains("id=\"custom-social-link\""));
    assert!(no_external_images.contains("src=\"/diagram.png\""));
    assert!(!no_external_images.contains("id=\"remote-image\""));
    assert!(!no_external_images.contains("https://cdn.example/remote.png"));
    assert!(!no_external_images.contains("id=\"social-image\""));
}
