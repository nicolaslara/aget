#[path = "cleanup/attributes.rs"]
mod attributes;
#[path = "cleanup/defaults.rs"]
mod defaults;
#[path = "cleanup/images.rs"]
mod images;
#[path = "cleanup/links.rs"]
mod links;
#[path = "cleanup/markdown.rs"]
mod markdown;
#[path = "cleanup/selection.rs"]
mod selection;
#[path = "cleanup/support.rs"]
mod support;

use std::path::Path;

use crate::support::mock_site::MockSite;

pub(super) fn assert_cleanup_outputs(aget_home: &Path, site: &MockSite) {
    defaults::assert_default_cleaned_html(aget_home, site);
    attributes::assert_attribute_cleanup_options(aget_home, site);
    images::assert_image_cleanup_options(aget_home, site);
    links::assert_link_cleanup_options(aget_home, site);
    selection::assert_selection_after_attribute_pruning(aget_home, site);
    markdown::assert_markdown_cleanup_output(aget_home, site);
}
