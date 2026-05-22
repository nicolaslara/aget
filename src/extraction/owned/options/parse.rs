use std::time::Duration;

use url::Url;

use crate::browser_cdp::PageWaitUntil;
use crate::error::AgetError;
use crate::extraction::extraction_failed;
use crate::extraction::html_clean::parse_css_selector;

pub(super) fn parse_owned_base_url(value: &str) -> Result<String, AgetError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(extraction_failed(
            "crawl4ai.base_url expects a non-empty URL",
        ));
    }
    Url::parse(value).map_err(|error| {
        extraction_failed(format!(
            "crawl4ai.base_url expects an absolute URL, got '{value}': {error}"
        ))
    })?;
    Ok(value.to_string())
}

pub(super) fn parse_owned_excluded_tags(value: &str) -> Result<Vec<String>, AgetError> {
    parse_owned_list("crawl4ai.excluded_tags", value)?
        .into_iter()
        .map(|tag| {
            if is_html_tag_name(&tag) {
                Ok(tag)
            } else {
                Err(extraction_failed(format!(
                    "crawl4ai.excluded_tags entry '{tag}' is not a plain HTML tag name"
                )))
            }
        })
        .collect()
}

pub(super) fn parse_owned_list(name: &str, value: &str) -> Result<Vec<String>, AgetError> {
    let items = value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if items.is_empty() {
        return Err(extraction_failed(format!(
            "{name} expects a comma-separated list with at least one entry"
        )));
    }
    Ok(items)
}

fn is_html_tag_name(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic())
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

pub(super) fn parse_owned_target_elements(value: &str) -> Result<Vec<String>, AgetError> {
    value
        .split(',')
        .map(str::trim)
        .filter(|selector| !selector.is_empty())
        .map(|selector| {
            parse_css_selector(selector)?;
            Ok(selector.to_string())
        })
        .collect()
}

pub(super) fn parse_owned_render_delay(value: &str) -> Result<Duration, AgetError> {
    parse_owned_seconds("crawl4ai.delay_before_return_html", value)
}

pub(super) fn parse_owned_seconds(name: &str, value: &str) -> Result<Duration, AgetError> {
    let seconds = value.trim().parse::<f64>().map_err(|_| {
        extraction_failed(format!(
            "{name} expects a non-negative number of seconds, got '{value}'"
        ))
    })?;
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(extraction_failed(format!(
            "{name} expects a non-negative finite number of seconds, got '{value}'"
        )));
    }
    Ok(Duration::from_secs_f64(seconds))
}

pub(super) fn parse_owned_bool(name: &str, value: &str) -> Result<bool, AgetError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(extraction_failed(format!(
            "{name} expects a boolean value, got '{value}'"
        ))),
    }
}

pub(super) fn parse_owned_milliseconds(name: &str, value: &str) -> Result<Duration, AgetError> {
    let milliseconds = value.trim().parse::<u64>().map_err(|_| {
        extraction_failed(format!(
            "{name} expects a non-negative integer number of milliseconds, got '{value}'"
        ))
    })?;
    Ok(Duration::from_millis(milliseconds))
}

pub(super) fn parse_owned_word_count_threshold(value: &str) -> Result<usize, AgetError> {
    value.trim().parse::<usize>().map_err(|_| {
        extraction_failed(format!(
            "crawl4ai.word_count_threshold expects a non-negative integer value, got '{value}'"
        ))
    })
}

pub(super) fn parse_owned_max_scroll_steps(value: &str) -> Result<usize, AgetError> {
    value.trim().parse::<usize>().map_err(|_| {
        extraction_failed(format!(
            "crawl4ai.max_scroll_steps expects a non-negative integer value, got '{value}'"
        ))
    })
}

pub(super) fn parse_owned_wait_until(value: &str) -> Result<PageWaitUntil, AgetError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "domcontentloaded" => Ok(PageWaitUntil::DomContentLoaded),
        "load" => Ok(PageWaitUntil::Load),
        "networkidle" => Ok(PageWaitUntil::NetworkIdle),
        _ => Err(extraction_failed(format!(
            "crawl4ai.wait_until supports only 'domcontentloaded', 'load', or 'networkidle' in the owned extractor, got '{value}'"
        ))),
    }
}

pub(super) fn validate_css_only_wait(value: &str) -> Result<(), AgetError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.starts_with("js:")
        || ["=>", "function(", "return ", ";"]
            .iter()
            .any(|marker| normalized.contains(marker))
    {
        return Err(extraction_failed(
            "--wait-for-selector only supports CSS selectors in v1; JavaScript wait conditions are not allowed",
        ));
    }
    Ok(())
}
