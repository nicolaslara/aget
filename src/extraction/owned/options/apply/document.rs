use crate::error::AgetError;

use super::super::parse::{
    parse_owned_base_url, parse_owned_bool, parse_owned_cache_mode, parse_owned_excluded_tags,
    parse_owned_list, parse_owned_target_elements,
};
use super::super::OwnedExtractorOptions;

pub(super) fn apply_document_option(
    owned_options: &mut OwnedExtractorOptions,
    full_key: &str,
    option_name: &str,
    value: &str,
) -> Result<bool, AgetError> {
    match option_name {
        "base_url" => {
            owned_options.base_url = Some(parse_owned_base_url(value)?);
        }
        "cache" | "cache_mode" => {
            parse_owned_cache_mode(full_key, value)?;
        }
        "css_selector" => {
            let selector = value.trim();
            if selector.is_empty() {
                owned_options.css_selector = None;
            } else {
                owned_options.css_selector = Some(selector.to_string());
            }
        }
        "excluded_tags" => {
            owned_options
                .excluded_tags
                .extend(parse_owned_excluded_tags(value)?);
        }
        "excluded_selector" => {
            let selector = value.trim();
            if !selector.is_empty() {
                owned_options.excluded_selectors.push(selector.to_string());
            }
        }
        "target_elements" => {
            owned_options
                .target_elements
                .extend(parse_owned_target_elements(value)?);
        }
        "only_text" => {
            owned_options.only_text = parse_owned_bool("crawl4ai.only_text", value)?;
        }
        "remove_consent_popups" => {
            owned_options.remove_consent_popups =
                parse_owned_bool("crawl4ai.remove_consent_popups", value)?;
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
        "keep_attrs" => {
            owned_options
                .keep_attrs
                .extend(parse_owned_list("crawl4ai.keep_attrs", value)?);
        }
        "prettiify" => {
            owned_options.prettiify = parse_owned_bool("crawl4ai.prettiify", value)?;
        }
        "user_agent" => {
            owned_options.user_agent = trimmed_string_option(value);
        }
        "locale" => {
            owned_options.locale = trimmed_string_option(value);
        }
        "timezone_id" => {
            owned_options.timezone_id = trimmed_string_option(value);
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn trimmed_string_option(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}
