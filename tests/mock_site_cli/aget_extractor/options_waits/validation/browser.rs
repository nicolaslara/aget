use std::path::Path;

use crate::support::mock_site::MockSite;

use super::support::assert_invalid_option_contains;

pub(super) fn assert_browser_option_validation(aget_home: &Path, site: &MockSite) {
    for (key, value, expected) in [
        (
            "crawl4ai.page_timeout",
            "soon",
            "crawl4ai.page_timeout expects a non-negative integer number of milliseconds",
        ),
        (
            "crawl4ai.wait_until",
            "commit",
            "crawl4ai.wait_until supports only 'domcontentloaded', 'load', or 'networkidle'",
        ),
        (
            "crawl4ai.wait_for_images",
            "eventually",
            "crawl4ai.wait_for_images expects a boolean value",
        ),
        (
            "crawl4ai.process_iframes",
            "maybe",
            "crawl4ai.process_iframes expects a boolean value",
        ),
        (
            "crawl4ai.scroll_delay",
            "later",
            "crawl4ai.scroll_delay expects a non-negative number of seconds",
        ),
        (
            "crawl4ai.max_scroll_steps",
            "many",
            "crawl4ai.max_scroll_steps expects a non-negative integer value",
        ),
    ] {
        assert_invalid_option_contains(aget_home, site, key, value, expected);
    }
}
