use crate::error::AgetError;
use crate::extraction::extraction_failed;

use super::parse::{
    parse_owned_base_url, parse_owned_body_width, parse_owned_bool, parse_owned_excluded_tags,
    parse_owned_list, parse_owned_max_scroll_steps, parse_owned_milliseconds,
    parse_owned_render_delay, parse_owned_seconds, parse_owned_target_elements,
    parse_owned_wait_until, parse_owned_word_count_threshold,
};
use super::OwnedExtractorOptions;

const SUPPORTED_OPTIONS: &str = "crawl4ai.base_url, crawl4ai.body_width, crawl4ai.bypass_tables, crawl4ai.close_quote, crawl4ai.default_image_alt, crawl4ai.delay_before_return_html, crawl4ai.emphasis_mark, crawl4ai.escape_snob, crawl4ai.exclude_all_images, crawl4ai.exclude_domains, crawl4ai.exclude_external_images, crawl4ai.exclude_external_links, crawl4ai.exclude_internal_links, crawl4ai.exclude_social_media_domains, crawl4ai.exclude_social_media_links, crawl4ai.excluded_tags, crawl4ai.flatten_shadow_dom, crawl4ai.ignore_emphasis, crawl4ai.ignore_images, crawl4ai.ignore_links, crawl4ai.ignore_mailto_links, crawl4ai.ignore_tables, crawl4ai.images_as_html, crawl4ai.images_to_alt, crawl4ai.images_with_size, crawl4ai.include_sup_sub, crawl4ai.keep_data_attributes, crawl4ai.mark_code, crawl4ai.max_scroll_steps, crawl4ai.only_text, crawl4ai.open_quote, crawl4ai.page_timeout, crawl4ai.process_iframes, crawl4ai.protect_links, crawl4ai.remove_forms, crawl4ai.remove_overlay_elements, crawl4ai.scan_full_page, crawl4ai.scroll_delay, crawl4ai.skip_internal_links, crawl4ai.strong_mark, crawl4ai.target_elements, crawl4ai.ul_item_mark, crawl4ai.use_automatic_links, crawl4ai.wait_for_images, crawl4ai.wait_for_timeout, crawl4ai.wait_until, crawl4ai.word_count_threshold";

pub(super) fn apply_owned_extractor_option(
    owned_options: &mut OwnedExtractorOptions,
    full_key: &str,
    option_name: &str,
    value: &str,
) -> Result<(), AgetError> {
    match option_name {
        "base_url" => {
            owned_options.base_url = Some(parse_owned_base_url(value)?);
        }
        "excluded_tags" => {
            owned_options
                .excluded_tags
                .extend(parse_owned_excluded_tags(value)?);
        }
        "target_elements" => {
            owned_options
                .target_elements
                .extend(parse_owned_target_elements(value)?);
        }
        "only_text" => {
            owned_options.only_text = parse_owned_bool("crawl4ai.only_text", value)?;
        }
        "remove_overlay_elements" => {
            owned_options.remove_overlay_elements =
                parse_owned_bool("crawl4ai.remove_overlay_elements", value)?;
        }
        "remove_forms" => {
            owned_options.remove_forms = parse_owned_bool("crawl4ai.remove_forms", value)?;
        }
        "keep_data_attributes" => {
            owned_options.keep_data_attributes =
                parse_owned_bool("crawl4ai.keep_data_attributes", value)?;
        }
        "exclude_all_images" => {
            owned_options.exclude_all_images =
                parse_owned_bool("crawl4ai.exclude_all_images", value)?;
        }
        "exclude_domains" => {
            owned_options
                .exclude_domains
                .extend(parse_owned_list("crawl4ai.exclude_domains", value)?);
        }
        "exclude_external_images" => {
            owned_options.exclude_external_images =
                parse_owned_bool("crawl4ai.exclude_external_images", value)?;
        }
        "exclude_external_links" => {
            owned_options.exclude_external_links =
                parse_owned_bool("crawl4ai.exclude_external_links", value)?;
        }
        "exclude_internal_links" => {
            owned_options.exclude_internal_links =
                parse_owned_bool("crawl4ai.exclude_internal_links", value)?;
        }
        "skip_internal_links" => {
            owned_options.skip_internal_links =
                parse_owned_bool("crawl4ai.skip_internal_links", value)?;
        }
        "default_image_alt" => {
            owned_options.default_image_alt = value.to_string();
        }
        "body_width" => {
            owned_options.body_width = parse_owned_body_width(value)?;
        }
        "open_quote" => {
            owned_options.open_quote = value.to_string();
        }
        "close_quote" => {
            owned_options.close_quote = value.to_string();
        }
        "ul_item_mark" => {
            owned_options.ul_item_mark = value.to_string();
        }
        "emphasis_mark" => {
            owned_options.emphasis_mark = value.to_string();
        }
        "strong_mark" => {
            owned_options.strong_mark = value.to_string();
        }
        "ignore_images" => {
            owned_options.ignore_images = parse_owned_bool("crawl4ai.ignore_images", value)?;
        }
        "images_as_html" => {
            owned_options.images_as_html = parse_owned_bool("crawl4ai.images_as_html", value)?;
        }
        "images_to_alt" => {
            owned_options.images_to_alt = parse_owned_bool("crawl4ai.images_to_alt", value)?;
        }
        "images_with_size" => {
            owned_options.images_with_size = parse_owned_bool("crawl4ai.images_with_size", value)?;
        }
        "ignore_emphasis" => {
            owned_options.ignore_emphasis = parse_owned_bool("crawl4ai.ignore_emphasis", value)?;
        }
        "ignore_links" => {
            owned_options.ignore_links = parse_owned_bool("crawl4ai.ignore_links", value)?;
        }
        "ignore_mailto_links" => {
            owned_options.ignore_mailto_links =
                parse_owned_bool("crawl4ai.ignore_mailto_links", value)?;
        }
        "ignore_tables" => {
            owned_options.ignore_tables = parse_owned_bool("crawl4ai.ignore_tables", value)?;
        }
        "bypass_tables" => {
            owned_options.bypass_tables = parse_owned_bool("crawl4ai.bypass_tables", value)?;
        }
        "protect_links" => {
            owned_options.protect_links = parse_owned_bool("crawl4ai.protect_links", value)?;
        }
        "use_automatic_links" => {
            owned_options.use_automatic_links =
                parse_owned_bool("crawl4ai.use_automatic_links", value)?;
        }
        "escape_snob" => {
            owned_options.escape_snob = parse_owned_bool("crawl4ai.escape_snob", value)?;
        }
        "include_sup_sub" => {
            owned_options.include_sup_sub = parse_owned_bool("crawl4ai.include_sup_sub", value)?;
        }
        "mark_code" => {
            parse_owned_bool("crawl4ai.mark_code", value)?;
            // Crawl4AI's active CustomHTML2Text path emits inline/fenced code
            // regardless of this option, so the owned renderer accepts it as a
            // validated compatibility no-op.
        }
        "exclude_social_media_domains" => {
            owned_options
                .exclude_social_media_domains
                .extend(parse_owned_list(
                    "crawl4ai.exclude_social_media_domains",
                    value,
                )?);
        }
        "exclude_social_media_links" => {
            owned_options.exclude_social_media_links =
                parse_owned_bool("crawl4ai.exclude_social_media_links", value)?;
        }
        "process_iframes" => {
            owned_options.process_iframes = parse_owned_bool("crawl4ai.process_iframes", value)?;
        }
        "word_count_threshold" => {
            owned_options.word_count_threshold = parse_owned_word_count_threshold(value)?;
        }
        "delay_before_return_html" => {
            owned_options.render_settle_delay = parse_owned_render_delay(value)?;
        }
        "page_timeout" => {
            owned_options.page_timeout =
                Some(parse_owned_milliseconds("crawl4ai.page_timeout", value)?);
        }
        "wait_for_timeout" => {
            owned_options.wait_for_timeout = Some(parse_owned_milliseconds(
                "crawl4ai.wait_for_timeout",
                value,
            )?);
        }
        "wait_until" => {
            owned_options.wait_until = parse_owned_wait_until(value)?;
        }
        "wait_for_images" => {
            owned_options.wait_for_images = parse_owned_bool("crawl4ai.wait_for_images", value)?;
        }
        "scan_full_page" => {
            owned_options.scan_full_page = parse_owned_bool("crawl4ai.scan_full_page", value)?;
        }
        "scroll_delay" => {
            owned_options.scroll_delay = parse_owned_seconds("crawl4ai.scroll_delay", value)?;
        }
        "max_scroll_steps" => {
            owned_options.max_scroll_steps = parse_owned_max_scroll_steps(value)?;
        }
        "flatten_shadow_dom" => {
            owned_options.flatten_shadow_dom =
                parse_owned_bool("crawl4ai.flatten_shadow_dom", value)?;
        }
        _ => {
            return Err(extraction_failed(format!(
                "owned extractor does not support backend option '{full_key}'; supported options: {SUPPORTED_OPTIONS}"
            )));
        }
    }
    Ok(())
}
