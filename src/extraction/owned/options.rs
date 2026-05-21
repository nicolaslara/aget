use std::time::Duration;

use crate::browser_cdp::PageWaitUntil;
use crate::error::AgetError;

use crate::extraction::html_clean::parse_css_selector;
use crate::extraction::{extraction_failed, GetOptions};

const DEFAULT_RENDER_SETTLE_DELAY: Duration = Duration::from_millis(100);
const DEFAULT_FULL_PAGE_SCROLL_DELAY: Duration = Duration::from_millis(200);
const DEFAULT_FULL_PAGE_MAX_SCROLL_STEPS: usize = 10;

#[derive(Debug)]
pub(crate) struct OwnedExtractorOptions {
    pub(crate) excluded_tags: Vec<String>,
    pub(crate) target_elements: Vec<String>,
    pub(crate) only_text: bool,
    pub(crate) remove_forms: bool,
    pub(crate) keep_data_attributes: bool,
    pub(crate) exclude_all_images: bool,
    pub(crate) wait_until: PageWaitUntil,
    pub(crate) wait_for_images: bool,
    pub(crate) scan_full_page: bool,
    pub(crate) scroll_delay: Duration,
    pub(crate) max_scroll_steps: usize,
    pub(crate) flatten_shadow_dom: bool,
    pub(crate) word_count_threshold: usize,
    pub(crate) render_settle_delay: Duration,
    pub(crate) page_timeout: Option<Duration>,
    pub(crate) wait_for_timeout: Option<Duration>,
}

impl Default for OwnedExtractorOptions {
    fn default() -> Self {
        Self {
            excluded_tags: Vec::new(),
            target_elements: Vec::new(),
            only_text: false,
            remove_forms: false,
            keep_data_attributes: false,
            exclude_all_images: false,
            wait_until: PageWaitUntil::Load,
            wait_for_images: false,
            scan_full_page: false,
            scroll_delay: DEFAULT_FULL_PAGE_SCROLL_DELAY,
            max_scroll_steps: DEFAULT_FULL_PAGE_MAX_SCROLL_STEPS,
            flatten_shadow_dom: false,
            word_count_threshold: 1,
            render_settle_delay: DEFAULT_RENDER_SETTLE_DELAY,
            page_timeout: None,
            wait_for_timeout: None,
        }
    }
}

pub(crate) fn validate_owned_extraction_options(
    options: &GetOptions,
) -> Result<OwnedExtractorOptions, AgetError> {
    if let Some(wait_for) = &options.wait_for_selector {
        validate_css_only_wait(wait_for)?;
    }
    let mut owned_options = OwnedExtractorOptions::default();
    for option in &options.backend_options {
        let option_name = option.key.strip_prefix("crawl4ai.").ok_or_else(|| {
            extraction_failed(format!(
                "owned extractor backend option '{}' must use the crawl4ai namespace",
                option.key
            ))
        })?;
        match option_name {
            "excluded_tags" => {
                owned_options
                    .excluded_tags
                    .extend(parse_owned_excluded_tags(&option.value)?);
            }
            "target_elements" => {
                owned_options
                    .target_elements
                    .extend(parse_owned_target_elements(&option.value)?);
            }
            "only_text" => {
                owned_options.only_text = parse_owned_bool("crawl4ai.only_text", &option.value)?;
            }
            "remove_forms" => {
                owned_options.remove_forms =
                    parse_owned_bool("crawl4ai.remove_forms", &option.value)?;
            }
            "keep_data_attributes" => {
                owned_options.keep_data_attributes =
                    parse_owned_bool("crawl4ai.keep_data_attributes", &option.value)?;
            }
            "exclude_all_images" => {
                owned_options.exclude_all_images =
                    parse_owned_bool("crawl4ai.exclude_all_images", &option.value)?;
            }
            "word_count_threshold" => {
                owned_options.word_count_threshold =
                    parse_owned_word_count_threshold(&option.value)?;
            }
            "delay_before_return_html" => {
                owned_options.render_settle_delay = parse_owned_render_delay(&option.value)?;
            }
            "page_timeout" => {
                owned_options.page_timeout = Some(parse_owned_milliseconds(
                    "crawl4ai.page_timeout",
                    &option.value,
                )?);
            }
            "wait_for_timeout" => {
                owned_options.wait_for_timeout = Some(parse_owned_milliseconds(
                    "crawl4ai.wait_for_timeout",
                    &option.value,
                )?);
            }
            "wait_until" => {
                owned_options.wait_until = parse_owned_wait_until(&option.value)?;
            }
            "wait_for_images" => {
                owned_options.wait_for_images =
                    parse_owned_bool("crawl4ai.wait_for_images", &option.value)?;
            }
            "scan_full_page" => {
                owned_options.scan_full_page =
                    parse_owned_bool("crawl4ai.scan_full_page", &option.value)?;
            }
            "scroll_delay" => {
                owned_options.scroll_delay =
                    parse_owned_seconds("crawl4ai.scroll_delay", &option.value)?;
            }
            "max_scroll_steps" => {
                owned_options.max_scroll_steps = parse_owned_max_scroll_steps(&option.value)?;
            }
            "flatten_shadow_dom" => {
                owned_options.flatten_shadow_dom =
                    parse_owned_bool("crawl4ai.flatten_shadow_dom", &option.value)?;
            }
            _ => {
                return Err(extraction_failed(format!(
                    "owned extractor does not support backend option '{}'; supported options: crawl4ai.delay_before_return_html, crawl4ai.exclude_all_images, crawl4ai.excluded_tags, crawl4ai.flatten_shadow_dom, crawl4ai.keep_data_attributes, crawl4ai.max_scroll_steps, crawl4ai.only_text, crawl4ai.page_timeout, crawl4ai.remove_forms, crawl4ai.scan_full_page, crawl4ai.scroll_delay, crawl4ai.target_elements, crawl4ai.wait_for_images, crawl4ai.wait_for_timeout, crawl4ai.wait_until, crawl4ai.word_count_threshold",
                    option.key
                )));
            }
        }
    }
    Ok(owned_options)
}

fn parse_owned_excluded_tags(value: &str) -> Result<Vec<String>, AgetError> {
    value
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(|tag| {
            if is_html_tag_name(tag) {
                Ok(tag.to_string())
            } else {
                Err(extraction_failed(format!(
                    "crawl4ai.excluded_tags entry '{tag}' is not a plain HTML tag name"
                )))
            }
        })
        .collect()
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

fn parse_owned_target_elements(value: &str) -> Result<Vec<String>, AgetError> {
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

fn parse_owned_render_delay(value: &str) -> Result<Duration, AgetError> {
    parse_owned_seconds("crawl4ai.delay_before_return_html", value)
}

fn parse_owned_seconds(name: &str, value: &str) -> Result<Duration, AgetError> {
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

fn parse_owned_bool(name: &str, value: &str) -> Result<bool, AgetError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(extraction_failed(format!(
            "{name} expects a boolean value, got '{value}'"
        ))),
    }
}

fn parse_owned_milliseconds(name: &str, value: &str) -> Result<Duration, AgetError> {
    let milliseconds = value.trim().parse::<u64>().map_err(|_| {
        extraction_failed(format!(
            "{name} expects a non-negative integer number of milliseconds, got '{value}'"
        ))
    })?;
    Ok(Duration::from_millis(milliseconds))
}

fn parse_owned_word_count_threshold(value: &str) -> Result<usize, AgetError> {
    value.trim().parse::<usize>().map_err(|_| {
        extraction_failed(format!(
            "crawl4ai.word_count_threshold expects a non-negative integer value, got '{value}'"
        ))
    })
}

fn parse_owned_max_scroll_steps(value: &str) -> Result<usize, AgetError> {
    value.trim().parse::<usize>().map_err(|_| {
        extraction_failed(format!(
            "crawl4ai.max_scroll_steps expects a non-negative integer value, got '{value}'"
        ))
    })
}

fn parse_owned_wait_until(value: &str) -> Result<PageWaitUntil, AgetError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "domcontentloaded" => Ok(PageWaitUntil::DomContentLoaded),
        "load" => Ok(PageWaitUntil::Load),
        "networkidle" => Ok(PageWaitUntil::NetworkIdle),
        _ => Err(extraction_failed(format!(
            "crawl4ai.wait_until supports only 'domcontentloaded', 'load', or 'networkidle' in the owned extractor, got '{value}'"
        ))),
    }
}

fn validate_css_only_wait(value: &str) -> Result<(), AgetError> {
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
