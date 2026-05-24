use std::path::Path;

use crate::support::mock_site::MockSite;

use super::support::html_cleanup;

pub(super) fn assert_link_cleanup_options(aget_home: &Path, site: &MockSite) {
    assert_external_link_cleanup(aget_home, site);
    assert_internal_link_cleanup(aget_home, site);
    assert_social_link_cleanup(aget_home, site);
    assert_domain_cleanup(aget_home, site);
}

fn assert_external_link_cleanup(aget_home: &Path, site: &MockSite) {
    let no_external_links =
        html_cleanup(aget_home, site, &[("aget.exclude_external_links", "true")]);
    assert!(no_external_links.contains("href=\"/kept\""));
    assert!(no_external_links.contains("id=\"same-domain-link\""));
    assert!(!no_external_links.contains("id=\"external-link\""));
    assert!(!no_external_links.contains("https://external.example/out"));
    assert!(!no_external_links.contains("id=\"social-link\""));
    assert!(!no_external_links.contains("id=\"custom-social-link\""));
    assert!(no_external_links.contains("id=\"remote-image\""));
    assert!(no_external_links.contains("id=\"social-image\""));
}

fn assert_internal_link_cleanup(aget_home: &Path, site: &MockSite) {
    let no_internal_links =
        html_cleanup(aget_home, site, &[("aget.exclude_internal_links", "true")]);
    assert!(!no_internal_links.contains("id=\"kept-link\""));
    assert!(!no_internal_links.contains("id=\"same-domain-link\""));
    assert!(!no_internal_links.contains("href=\"/same-domain\""));
    assert!(!no_internal_links.contains("id=\"empty-anchor\""));
    assert!(no_internal_links.contains("id=\"external-link\""));
    assert!(no_internal_links.contains("https://external.example/out"));
    assert!(no_internal_links.contains("id=\"mailto-link\""));
    assert!(no_internal_links.contains("href=\"mailto:help@example.com\""));
    assert!(no_internal_links.contains("id=\"social-link\""));
    assert!(no_internal_links.contains("id=\"custom-social-link\""));
    assert!(no_internal_links.contains("id=\"remote-image\""));
}

fn assert_social_link_cleanup(aget_home: &Path, site: &MockSite) {
    let no_social_links = html_cleanup(
        aget_home,
        site,
        &[("aget.exclude_social_media_links", "true")],
    );
    assert!(no_social_links.contains("href=\"/kept\""));
    assert!(no_social_links.contains("id=\"external-link\""));
    assert!(no_social_links.contains("id=\"custom-social-link\""));
    assert!(no_social_links.contains("id=\"remote-image\""));
    assert!(no_social_links.contains("id=\"social-image\""));
    assert!(!no_social_links.contains("id=\"social-link\""));
    assert!(!no_social_links.contains("https://www.linkedin.com/company/aget"));

    let no_custom_social_links = html_cleanup(
        aget_home,
        site,
        &[
            ("aget.exclude_social_media_links", "true"),
            ("aget.exclude_social_media_domains", "social.example"),
        ],
    );
    assert!(no_custom_social_links.contains("href=\"/kept\""));
    assert!(no_custom_social_links.contains("id=\"external-link\""));
    assert!(no_custom_social_links.contains("id=\"remote-image\""));
    assert!(no_custom_social_links.contains("id=\"social-image\""));
    assert!(!no_custom_social_links.contains("id=\"social-link\""));
    assert!(!no_custom_social_links.contains("https://www.linkedin.com/company/aget"));
    assert!(!no_custom_social_links.contains("id=\"custom-social-link\""));
    assert!(!no_custom_social_links.contains("https://social.example/company/aget"));
}

fn assert_domain_cleanup(aget_home: &Path, site: &MockSite) {
    let no_excluded_domains = html_cleanup(
        aget_home,
        site,
        &[("aget.exclude_domains", "external.example,cdn.example")],
    );
    assert!(no_excluded_domains.contains("href=\"/kept\""));
    assert!(no_excluded_domains.contains("src=\"/diagram.png\""));
    assert!(!no_excluded_domains.contains("id=\"external-link\""));
    assert!(!no_excluded_domains.contains("https://external.example/out"));
    assert!(!no_excluded_domains.contains("id=\"remote-image\""));
    assert!(!no_excluded_domains.contains("https://cdn.example/remote.png"));
}
