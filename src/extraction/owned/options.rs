use std::time::Duration;

use crate::browser_cdp::PageWaitUntil;
use crate::error::AgetError;
use crate::extraction::{extraction_failed, GetOptions};

mod apply;
mod parse;

use apply::apply_owned_extractor_option;
use parse::validate_css_only_wait;

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
    pub(crate) default_image_alt: String,
    pub(crate) body_width: usize,
    pub(crate) open_quote: String,
    pub(crate) close_quote: String,
    pub(crate) ul_item_mark: String,
    pub(crate) emphasis_mark: String,
    pub(crate) strong_mark: String,
    pub(crate) skip_internal_links: bool,
    pub(crate) ignore_images: bool,
    pub(crate) images_as_html: bool,
    pub(crate) images_to_alt: bool,
    pub(crate) images_with_size: bool,
    pub(crate) ignore_emphasis: bool,
    pub(crate) ignore_links: bool,
    pub(crate) inline_links: bool,
    pub(crate) ignore_mailto_links: bool,
    pub(crate) ignore_tables: bool,
    pub(crate) bypass_tables: bool,
    pub(crate) hide_strikethrough: bool,
    pub(crate) pad_tables: bool,
    pub(crate) protect_links: bool,
    pub(crate) use_automatic_links: bool,
    pub(crate) wrap_links: bool,
    pub(crate) wrap_list_items: bool,
    pub(crate) wrap_tables: bool,
    pub(crate) unicode_snob: bool,
    pub(crate) escape_snob: bool,
    pub(crate) include_sup_sub: bool,
    pub(crate) single_line_break: bool,
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
            default_image_alt: String::new(),
            body_width: 0,
            open_quote: "\"".to_string(),
            close_quote: "\"".to_string(),
            ul_item_mark: "*".to_string(),
            emphasis_mark: "_".to_string(),
            strong_mark: "**".to_string(),
            skip_internal_links: false,
            ignore_images: false,
            images_as_html: false,
            images_to_alt: false,
            images_with_size: false,
            ignore_emphasis: false,
            ignore_links: false,
            inline_links: true,
            ignore_mailto_links: true,
            ignore_tables: false,
            bypass_tables: false,
            hide_strikethrough: false,
            pad_tables: false,
            protect_links: false,
            use_automatic_links: true,
            wrap_links: true,
            wrap_list_items: false,
            wrap_tables: false,
            unicode_snob: true,
            escape_snob: false,
            include_sup_sub: false,
            single_line_break: false,
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
        apply_owned_extractor_option(&mut owned_options, &option.key, option_name, &option.value)?;
    }
    Ok(owned_options)
}
