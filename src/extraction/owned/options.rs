use std::time::Duration;

use crate::browser_cdp::PageWaitUntil;
use crate::error::AgetError;
use crate::extraction::{extraction_failed, GetOptions};

mod parse;

use parse::{
    parse_owned_base_url, parse_owned_bool, parse_owned_excluded_tags, parse_owned_list,
    parse_owned_max_scroll_steps, parse_owned_milliseconds, parse_owned_render_delay,
    parse_owned_seconds, parse_owned_target_elements, parse_owned_wait_until,
    parse_owned_word_count_threshold, validate_css_only_wait,
};

const DEFAULT_RENDER_SETTLE_DELAY: Duration = Duration::from_millis(100);
const DEFAULT_FULL_PAGE_SCROLL_DELAY: Duration = Duration::from_millis(200);
const DEFAULT_FULL_PAGE_MAX_SCROLL_STEPS: usize = 10;

#[derive(Debug)]
pub(crate) struct OwnedExtractorOptions {
    pub(crate) base_url: Option<String>,
    pub(crate) excluded_tags: Vec<String>,
    pub(crate) target_elements: Vec<String>,
    pub(crate) only_text: bool,
    pub(crate) remove_overlay_elements: bool,
    pub(crate) remove_forms: bool,
    pub(crate) keep_data_attributes: bool,
    pub(crate) exclude_all_images: bool,
    pub(crate) exclude_domains: Vec<String>,
    pub(crate) exclude_external_images: bool,
    pub(crate) exclude_external_links: bool,
    pub(crate) exclude_internal_links: bool,
    pub(crate) skip_internal_links: bool,
    pub(crate) ignore_images: bool,
    pub(crate) images_as_html: bool,
    pub(crate) images_to_alt: bool,
    pub(crate) images_with_size: bool,
    pub(crate) ignore_emphasis: bool,
    pub(crate) ignore_links: bool,
    pub(crate) ignore_mailto_links: bool,
    pub(crate) ignore_tables: bool,
    pub(crate) bypass_tables: bool,
    pub(crate) protect_links: bool,
    pub(crate) use_automatic_links: bool,
    pub(crate) escape_snob: bool,
    pub(crate) include_sup_sub: bool,
    pub(crate) exclude_social_media_domains: Vec<String>,
    pub(crate) exclude_social_media_links: bool,
    pub(crate) process_iframes: bool,
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
            base_url: None,
            excluded_tags: Vec::new(),
            target_elements: Vec::new(),
            only_text: false,
            remove_overlay_elements: true,
            remove_forms: false,
            keep_data_attributes: false,
            exclude_all_images: false,
            exclude_domains: Vec::new(),
            exclude_external_images: false,
            exclude_external_links: false,
            exclude_internal_links: false,
            skip_internal_links: false,
            ignore_images: false,
            images_as_html: false,
            images_to_alt: false,
            images_with_size: false,
            ignore_emphasis: false,
            ignore_links: false,
            ignore_mailto_links: true,
            ignore_tables: false,
            bypass_tables: false,
            protect_links: false,
            use_automatic_links: true,
            escape_snob: false,
            include_sup_sub: false,
            exclude_social_media_domains: Vec::new(),
            exclude_social_media_links: false,
            process_iframes: false,
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
            "base_url" => {
                owned_options.base_url = Some(parse_owned_base_url(&option.value)?);
            }
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
            "remove_overlay_elements" => {
                owned_options.remove_overlay_elements =
                    parse_owned_bool("crawl4ai.remove_overlay_elements", &option.value)?;
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
            "exclude_domains" => {
                owned_options
                    .exclude_domains
                    .extend(parse_owned_list("crawl4ai.exclude_domains", &option.value)?);
            }
            "exclude_external_images" => {
                owned_options.exclude_external_images =
                    parse_owned_bool("crawl4ai.exclude_external_images", &option.value)?;
            }
            "exclude_external_links" => {
                owned_options.exclude_external_links =
                    parse_owned_bool("crawl4ai.exclude_external_links", &option.value)?;
            }
            "exclude_internal_links" => {
                owned_options.exclude_internal_links =
                    parse_owned_bool("crawl4ai.exclude_internal_links", &option.value)?;
            }
            "skip_internal_links" => {
                owned_options.skip_internal_links =
                    parse_owned_bool("crawl4ai.skip_internal_links", &option.value)?;
            }
            "ignore_images" => {
                owned_options.ignore_images =
                    parse_owned_bool("crawl4ai.ignore_images", &option.value)?;
            }
            "images_as_html" => {
                owned_options.images_as_html =
                    parse_owned_bool("crawl4ai.images_as_html", &option.value)?;
            }
            "images_to_alt" => {
                owned_options.images_to_alt =
                    parse_owned_bool("crawl4ai.images_to_alt", &option.value)?;
            }
            "images_with_size" => {
                owned_options.images_with_size =
                    parse_owned_bool("crawl4ai.images_with_size", &option.value)?;
            }
            "ignore_emphasis" => {
                owned_options.ignore_emphasis =
                    parse_owned_bool("crawl4ai.ignore_emphasis", &option.value)?;
            }
            "ignore_links" => {
                owned_options.ignore_links =
                    parse_owned_bool("crawl4ai.ignore_links", &option.value)?;
            }
            "ignore_mailto_links" => {
                owned_options.ignore_mailto_links =
                    parse_owned_bool("crawl4ai.ignore_mailto_links", &option.value)?;
            }
            "ignore_tables" => {
                owned_options.ignore_tables =
                    parse_owned_bool("crawl4ai.ignore_tables", &option.value)?;
            }
            "bypass_tables" => {
                owned_options.bypass_tables =
                    parse_owned_bool("crawl4ai.bypass_tables", &option.value)?;
            }
            "protect_links" => {
                owned_options.protect_links =
                    parse_owned_bool("crawl4ai.protect_links", &option.value)?;
            }
            "use_automatic_links" => {
                owned_options.use_automatic_links =
                    parse_owned_bool("crawl4ai.use_automatic_links", &option.value)?;
            }
            "escape_snob" => {
                owned_options.escape_snob =
                    parse_owned_bool("crawl4ai.escape_snob", &option.value)?;
            }
            "include_sup_sub" => {
                owned_options.include_sup_sub =
                    parse_owned_bool("crawl4ai.include_sup_sub", &option.value)?;
            }
            "exclude_social_media_domains" => {
                owned_options
                    .exclude_social_media_domains
                    .extend(parse_owned_list(
                        "crawl4ai.exclude_social_media_domains",
                        &option.value,
                    )?);
            }
            "exclude_social_media_links" => {
                owned_options.exclude_social_media_links =
                    parse_owned_bool("crawl4ai.exclude_social_media_links", &option.value)?;
            }
            "process_iframes" => {
                owned_options.process_iframes =
                    parse_owned_bool("crawl4ai.process_iframes", &option.value)?;
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
                    "owned extractor does not support backend option '{}'; supported options: crawl4ai.base_url, crawl4ai.bypass_tables, crawl4ai.delay_before_return_html, crawl4ai.escape_snob, crawl4ai.exclude_all_images, crawl4ai.exclude_domains, crawl4ai.exclude_external_images, crawl4ai.exclude_external_links, crawl4ai.exclude_internal_links, crawl4ai.exclude_social_media_domains, crawl4ai.exclude_social_media_links, crawl4ai.excluded_tags, crawl4ai.flatten_shadow_dom, crawl4ai.ignore_emphasis, crawl4ai.ignore_images, crawl4ai.ignore_links, crawl4ai.ignore_mailto_links, crawl4ai.ignore_tables, crawl4ai.images_as_html, crawl4ai.images_to_alt, crawl4ai.images_with_size, crawl4ai.include_sup_sub, crawl4ai.keep_data_attributes, crawl4ai.max_scroll_steps, crawl4ai.only_text, crawl4ai.page_timeout, crawl4ai.process_iframes, crawl4ai.protect_links, crawl4ai.remove_forms, crawl4ai.remove_overlay_elements, crawl4ai.scan_full_page, crawl4ai.scroll_delay, crawl4ai.skip_internal_links, crawl4ai.target_elements, crawl4ai.use_automatic_links, crawl4ai.wait_for_images, crawl4ai.wait_for_timeout, crawl4ai.wait_until, crawl4ai.word_count_threshold",
                    option.key
                )));
            }
        }
    }
    Ok(owned_options)
}
