use crate::support::mock_site::MockSite;

pub(super) struct LinkImageUrls {
    pub(super) release: String,
    pub(super) diagram: String,
    pub(super) icon: String,
}

pub(super) fn link_image_urls(site: &MockSite) -> LinkImageUrls {
    LinkImageUrls {
        release: escaped_site_url(site, "/release(2026)"),
        diagram: escaped_site_url(site, "/assets/diagram(1).png"),
        icon: escaped_site_url(site, "/icons/app(1).svg"),
    }
}

fn escaped_site_url(site: &MockSite, path: &str) -> String {
    site.url(path).replace('(', "\\(").replace(')', "\\)")
}
