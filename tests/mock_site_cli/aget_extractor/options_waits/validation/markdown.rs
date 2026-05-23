use std::path::Path;

use crate::support::mock_site::MockSite;

use super::support::assert_invalid_option_contains;

pub(super) fn assert_markdown_option_validation(aget_home: &Path, site: &MockSite) {
    for (key, value, expected) in [
        (
            "crawl4ai.body_width",
            "wide",
            "crawl4ai.body_width expects a non-negative integer value",
        ),
        (
            "crawl4ai.mark_code",
            "maybe",
            "crawl4ai.mark_code expects a boolean value",
        ),
        (
            "crawl4ai.handle_code_in_pre",
            "maybe",
            "crawl4ai.handle_code_in_pre expects a boolean value",
        ),
        (
            "crawl4ai.single_line_break",
            "sometimes",
            "crawl4ai.single_line_break expects a boolean value",
        ),
        (
            "crawl4ai.unicode_snob",
            "maybe",
            "crawl4ai.unicode_snob expects a boolean value",
        ),
        (
            "crawl4ai.wrap_links",
            "maybe",
            "crawl4ai.wrap_links expects a boolean value",
        ),
        (
            "crawl4ai.inline_links",
            "maybe",
            "crawl4ai.inline_links expects a boolean value",
        ),
        (
            "crawl4ai.ignore_anchors",
            "maybe",
            "crawl4ai.ignore_anchors expects a boolean value",
        ),
        (
            "crawl4ai.links_each_paragraph",
            "maybe",
            "crawl4ai.links_each_paragraph expects a boolean value",
        ),
        (
            "crawl4ai.wrap_list_items",
            "maybe",
            "crawl4ai.wrap_list_items expects a boolean value",
        ),
        (
            "crawl4ai.wrap_tables",
            "maybe",
            "crawl4ai.wrap_tables expects a boolean value",
        ),
        (
            "crawl4ai.pad_tables",
            "maybe",
            "crawl4ai.pad_tables expects a boolean value",
        ),
        (
            "crawl4ai.hide_strikethrough",
            "maybe",
            "crawl4ai.hide_strikethrough expects a boolean value",
        ),
        (
            "crawl4ai.google_doc",
            "maybe",
            "crawl4ai.google_doc expects a boolean value",
        ),
        (
            "crawl4ai.google_list_indent",
            "0",
            "crawl4ai.google_list_indent expects a positive integer pixel value",
        ),
    ] {
        assert_invalid_option_contains(aget_home, site, key, value, expected);
    }
}
