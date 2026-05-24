use std::path::Path;

use crate::support::mock_site::MockSite;

use super::support::assert_invalid_option_contains;

pub(super) fn assert_browser_option_validation(aget_home: &Path, site: &MockSite) {
    for (key, value, expected) in [
        (
            "aget.page_timeout",
            "soon",
            "aget.page_timeout expects a non-negative integer number of milliseconds",
        ),
        (
            "aget.wait_until",
            "commit",
            "aget.wait_until supports only 'domcontentloaded', 'load', or 'networkidle'",
        ),
        (
            "aget.wait_for_images",
            "eventually",
            "aget.wait_for_images expects a boolean value",
        ),
        (
            "aget.process_iframes",
            "maybe",
            "aget.process_iframes expects a boolean value",
        ),
        (
            "aget.scroll_delay",
            "later",
            "aget.scroll_delay expects a non-negative number of seconds",
        ),
        (
            "aget.max_scroll_steps",
            "many",
            "aget.max_scroll_steps expects a non-negative integer value",
        ),
    ] {
        assert_invalid_option_contains(aget_home, site, key, value, expected);
    }
}
