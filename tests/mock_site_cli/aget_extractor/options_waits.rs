#[path = "options_waits/content.rs"]
mod content;
#[path = "options_waits/validation.rs"]
mod validation;
#[path = "options_waits/waits.rs"]
mod waits;

use std::path::Path;

use crate::support::mock_site::MockSite;

pub(super) fn assert_backend_options_redirects_and_waits(aget_home: &Path, site: &MockSite) {
    content::assert_content_options(aget_home, site);
    validation::assert_option_validation(aget_home, site);
    waits::assert_redirects_and_waits(aget_home, site);
}
