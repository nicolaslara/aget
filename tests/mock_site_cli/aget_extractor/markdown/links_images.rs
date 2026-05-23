#[path = "links_images/defaults.rs"]
mod defaults;
#[path = "links_images/images.rs"]
mod images;
#[path = "links_images/links.rs"]
mod links;
#[path = "links_images/references.rs"]
mod references;
#[path = "links_images/support.rs"]
mod support;

use std::path::Path;

use crate::support::mock_site::MockSite;

pub(super) fn assert_links_and_images(aget_home: &Path, site: &MockSite) {
    let urls = support::link_image_urls(site);

    defaults::assert_default_links_and_images(aget_home, site, &urls);
    links::assert_automatic_links_option(aget_home, site);
    images::assert_image_options(aget_home, site, &urls);
    links::assert_link_options(aget_home, site, &urls);
    references::assert_reference_link_options(aget_home, site);
}
