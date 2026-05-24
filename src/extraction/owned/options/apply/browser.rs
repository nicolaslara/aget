use crate::error::AgetError;

use super::super::parse::{
    parse_owned_bool, parse_owned_max_scroll_steps, parse_owned_milliseconds,
    parse_owned_render_delay, parse_owned_seconds, parse_owned_wait_until,
    parse_owned_word_count_threshold,
};
use super::super::OwnedExtractorOptions;

pub(super) fn apply_browser_option(
    owned_options: &mut OwnedExtractorOptions,
    option_name: &str,
    value: &str,
) -> Result<bool, AgetError> {
    match option_name {
        "process_in_browser" => {
            owned_options.process_in_browser = parse_owned_bool("aget.process_in_browser", value)?;
        }
        "process_iframes" => {
            owned_options.process_iframes = parse_owned_bool("aget.process_iframes", value)?;
        }
        "word_count_threshold" => {
            owned_options.word_count_threshold = parse_owned_word_count_threshold(value)?;
        }
        "delay_before_return_html" => {
            owned_options.render_settle_delay = parse_owned_render_delay(value)?;
        }
        "page_timeout" => {
            owned_options.page_timeout =
                Some(parse_owned_milliseconds("aget.page_timeout", value)?);
        }
        "wait_for_timeout" => {
            owned_options.wait_for_timeout =
                Some(parse_owned_milliseconds("aget.wait_for_timeout", value)?);
        }
        "wait_until" => {
            owned_options.wait_until = parse_owned_wait_until(value)?;
        }
        "wait_for_images" => {
            owned_options.wait_for_images = parse_owned_bool("aget.wait_for_images", value)?;
        }
        "scan_full_page" => {
            owned_options.scan_full_page = parse_owned_bool("aget.scan_full_page", value)?;
        }
        "scroll_delay" => {
            owned_options.scroll_delay = parse_owned_seconds("aget.scroll_delay", value)?;
        }
        "max_scroll_steps" => {
            owned_options.max_scroll_steps = parse_owned_max_scroll_steps(value)?;
        }
        "flatten_shadow_dom" => {
            owned_options.flatten_shadow_dom = parse_owned_bool("aget.flatten_shadow_dom", value)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
