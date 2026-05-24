use std::path::Path;

use crate::support::mock_site::MockSite;

use super::support::assert_invalid_option_contains;

const SUPPORTED_OPTIONS_MESSAGE: &str = "supported options: aget.base_url, aget.body_width, aget.bypass_tables, aget.cache, aget.cache_mode, aget.close_quote, aget.css_selector, aget.default_image_alt, aget.delay_before_return_html, aget.emphasis_mark, aget.escape_backslash, aget.escape_dash, aget.escape_dot, aget.escape_plus, aget.escape_snob, aget.exclude_all_images, aget.exclude_domains, aget.exclude_external_images, aget.exclude_external_links, aget.exclude_internal_links, aget.exclude_social_media_domains, aget.exclude_social_media_links, aget.excluded_selector, aget.excluded_tags, aget.flatten_shadow_dom, aget.google_doc, aget.google_list_indent, aget.handle_code_in_pre, aget.hide_strikethrough, aget.ignore_anchors, aget.ignore_emphasis, aget.ignore_images, aget.ignore_links, aget.ignore_mailto_links, aget.ignore_tables, aget.images_as_html, aget.images_to_alt, aget.images_with_size, aget.include_sup_sub, aget.inline_links, aget.keep_attrs, aget.keep_data_attributes, aget.links_each_paragraph, aget.locale, aget.mark_code, aget.max_scroll_steps, aget.only_text, aget.open_quote, aget.pad_tables, aget.page_timeout, aget.preserve_tags, aget.prettiify, aget.process_iframes, aget.process_in_browser, aget.protect_links, aget.remove_consent_popups, aget.remove_forms, aget.remove_overlay_elements, aget.scan_full_page, aget.scroll_delay, aget.single_line_break, aget.skip_internal_links, aget.strong_mark, aget.target_elements, aget.timezone_id, aget.ul_item_mark, aget.unicode_snob, aget.use_automatic_links, aget.user_agent, aget.wait_for_images, aget.wait_for_timeout, aget.wait_until, aget.word_count_threshold, aget.wrap_links, aget.wrap_list_items, aget.wrap_tables";

pub(super) fn assert_document_option_validation(aget_home: &Path, site: &MockSite) {
    assert_invalid_option_contains(
        aget_home,
        site,
        "aget.word_count_threshold",
        "many",
        "aget.word_count_threshold expects a non-negative integer value",
    );
    assert_invalid_option_contains(
        aget_home,
        site,
        "aget.cache",
        "sometimes",
        "aget.cache supports only bypass, disabled, enabled, read_only, or write_only",
    );
    assert_invalid_option_contains(
        aget_home,
        site,
        "aget.remove_consent_popups",
        "maybe",
        "aget.remove_consent_popups expects a boolean value",
    );
    assert_invalid_option_contains(
        aget_home,
        site,
        "aget.base_url",
        "/relative/",
        "aget.base_url expects an absolute URL",
    );
    assert_invalid_option_contains(
        aget_home,
        site,
        "aget.magic",
        "value",
        SUPPORTED_OPTIONS_MESSAGE,
    );
}
