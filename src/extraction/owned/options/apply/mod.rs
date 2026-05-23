use crate::error::AgetError;
use crate::extraction::extraction_failed;

mod browser;
mod document;
mod markdown;

use browser::apply_browser_option;
use document::apply_document_option;
use markdown::apply_markdown_option;

use super::OwnedExtractorOptions;

const SUPPORTED_OPTIONS: &str = "crawl4ai.base_url, crawl4ai.body_width, crawl4ai.bypass_tables, crawl4ai.cache, crawl4ai.cache_mode, crawl4ai.close_quote, crawl4ai.css_selector, crawl4ai.default_image_alt, crawl4ai.delay_before_return_html, crawl4ai.emphasis_mark, crawl4ai.escape_backslash, crawl4ai.escape_dash, crawl4ai.escape_dot, crawl4ai.escape_plus, crawl4ai.escape_snob, crawl4ai.exclude_all_images, crawl4ai.exclude_domains, crawl4ai.exclude_external_images, crawl4ai.exclude_external_links, crawl4ai.exclude_internal_links, crawl4ai.exclude_social_media_domains, crawl4ai.exclude_social_media_links, crawl4ai.excluded_selector, crawl4ai.excluded_tags, crawl4ai.flatten_shadow_dom, crawl4ai.google_doc, crawl4ai.google_list_indent, crawl4ai.handle_code_in_pre, crawl4ai.hide_strikethrough, crawl4ai.ignore_anchors, crawl4ai.ignore_emphasis, crawl4ai.ignore_images, crawl4ai.ignore_links, crawl4ai.ignore_mailto_links, crawl4ai.ignore_tables, crawl4ai.images_as_html, crawl4ai.images_to_alt, crawl4ai.images_with_size, crawl4ai.include_sup_sub, crawl4ai.inline_links, crawl4ai.keep_attrs, crawl4ai.keep_data_attributes, crawl4ai.links_each_paragraph, crawl4ai.locale, crawl4ai.mark_code, crawl4ai.max_scroll_steps, crawl4ai.only_text, crawl4ai.open_quote, crawl4ai.pad_tables, crawl4ai.page_timeout, crawl4ai.preserve_tags, crawl4ai.prettiify, crawl4ai.process_iframes, crawl4ai.process_in_browser, crawl4ai.protect_links, crawl4ai.remove_consent_popups, crawl4ai.remove_forms, crawl4ai.remove_overlay_elements, crawl4ai.scan_full_page, crawl4ai.scroll_delay, crawl4ai.single_line_break, crawl4ai.skip_internal_links, crawl4ai.strong_mark, crawl4ai.target_elements, crawl4ai.timezone_id, crawl4ai.ul_item_mark, crawl4ai.unicode_snob, crawl4ai.use_automatic_links, crawl4ai.user_agent, crawl4ai.wait_for_images, crawl4ai.wait_for_timeout, crawl4ai.wait_until, crawl4ai.word_count_threshold, crawl4ai.wrap_links, crawl4ai.wrap_list_items, crawl4ai.wrap_tables";

pub(super) fn apply_owned_extractor_option(
    owned_options: &mut OwnedExtractorOptions,
    full_key: &str,
    option_name: &str,
    value: &str,
) -> Result<(), AgetError> {
    if apply_document_option(owned_options, full_key, option_name, value)? {
        return Ok(());
    }
    if apply_markdown_option(owned_options, option_name, value)? {
        return Ok(());
    }
    if apply_browser_option(owned_options, option_name, value)? {
        return Ok(());
    }
    Err(extraction_failed(format!(
        "owned extractor does not support backend option '{full_key}'; supported options: {SUPPORTED_OPTIONS}"
    )))
}
