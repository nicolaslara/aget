use std::path::Path;

use crate::support::mock_site::MockSite;

use super::support::html_cleanup;

pub(super) fn assert_attribute_cleanup_options(aget_home: &Path, site: &MockSite) {
    let data_attributes = html_cleanup(aget_home, site, &[("aget.keep_data_attributes", "true")]);
    assert!(data_attributes.contains("data-private=\"main-secret\""));
    assert!(data_attributes.contains("data-select=\"summary\""));
    assert!(data_attributes.contains("data-private=\"paragraph-secret\""));
    assert!(data_attributes.contains("data-private=\"link-secret\""));
    assert!(!data_attributes.contains("style="));
    assert!(!data_attributes.contains("onclick="));
    assert!(!data_attributes.contains("aria-label="));
    assert!(!data_attributes.contains("rel=\"nofollow\""));

    let kept_attrs = html_cleanup(aget_home, site, &[("aget.keep_attrs", "aria-label,rel")]);
    assert!(kept_attrs.contains("aria-label=\"private label\""));
    assert!(kept_attrs.contains("rel=\"nofollow\""));
    assert!(!kept_attrs.contains("data-private"));
    assert!(!kept_attrs.contains("style="));
    assert!(!kept_attrs.contains("onclick="));

    let prettified_html = html_cleanup(aget_home, site, &[("aget.prettiify", "true")]);
    assert!(prettified_html.contains("<h1>\n      Cleanup Main\n    </h1>"));
    assert!(prettified_html.contains("\n    <a class=\"cta\""));
}
