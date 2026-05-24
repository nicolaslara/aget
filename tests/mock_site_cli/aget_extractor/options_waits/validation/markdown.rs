use std::path::Path;

use crate::support::mock_site::MockSite;

use super::support::assert_invalid_option_contains;

pub(super) fn assert_markdown_option_validation(aget_home: &Path, site: &MockSite) {
    for (key, value, expected) in [
        (
            "aget.body_width",
            "wide",
            "aget.body_width expects a non-negative integer value",
        ),
        (
            "aget.mark_code",
            "maybe",
            "aget.mark_code expects a boolean value",
        ),
        (
            "aget.handle_code_in_pre",
            "maybe",
            "aget.handle_code_in_pre expects a boolean value",
        ),
        (
            "aget.single_line_break",
            "sometimes",
            "aget.single_line_break expects a boolean value",
        ),
        (
            "aget.unicode_snob",
            "maybe",
            "aget.unicode_snob expects a boolean value",
        ),
        (
            "aget.wrap_links",
            "maybe",
            "aget.wrap_links expects a boolean value",
        ),
        (
            "aget.inline_links",
            "maybe",
            "aget.inline_links expects a boolean value",
        ),
        (
            "aget.ignore_anchors",
            "maybe",
            "aget.ignore_anchors expects a boolean value",
        ),
        (
            "aget.links_each_paragraph",
            "maybe",
            "aget.links_each_paragraph expects a boolean value",
        ),
        (
            "aget.wrap_list_items",
            "maybe",
            "aget.wrap_list_items expects a boolean value",
        ),
        (
            "aget.wrap_tables",
            "maybe",
            "aget.wrap_tables expects a boolean value",
        ),
        (
            "aget.pad_tables",
            "maybe",
            "aget.pad_tables expects a boolean value",
        ),
        (
            "aget.hide_strikethrough",
            "maybe",
            "aget.hide_strikethrough expects a boolean value",
        ),
        (
            "aget.google_doc",
            "maybe",
            "aget.google_doc expects a boolean value",
        ),
        (
            "aget.google_list_indent",
            "0",
            "aget.google_list_indent expects a positive integer pixel value",
        ),
    ] {
        assert_invalid_option_contains(aget_home, site, key, value, expected);
    }
}
