use std::path::Path;
use std::time::Duration;

use ego_tree::NodeId;
use scraper::{ElementRef, Html};

use crate::browser_cdp::{BrowserRenderRequest, PageWaitUntil};
use crate::cli::OutputFormat;
use crate::error::{AgetError, ErrorCode};
use crate::session::PlaywrightState;

use super::artifacts::write_private_file;
use super::html_clean::{
    clean_owned_base64_image_sources, parse_css_selector, prune_owned_unwanted_attributes,
    remove_owned_empty_elements, remove_owned_excluded_tags, remove_owned_overlay_elements,
    remove_selected_elements,
};
use super::http::{owned_fetch, OwnedHttpResponse};
use super::markdown::{element_to_markdown, normalize_markdown, resolve_markdown_url};
use super::{
    extraction_failed, io_aget_error, BrowserFallbackRequest, BrowserFallbackResult,
    ExtractorBackendResult, ExtractorRequest, GetOptions,
};

pub(super) const OWNED_EXTRACTOR: &str = "aget-owned-extractor";

const OWNED_BROWSER_FALLBACK: &str = "aget-owned-browser-fallback";
const OWNED_FALLBACK_WARNING: &str = "aget-owned fallback used after primary extractor failed";
const DEFAULT_RENDER_SETTLE_DELAY: Duration = Duration::from_millis(100);

pub(super) fn run_owned_extractor_backend(
    request: ExtractorRequest<'_>,
) -> Result<ExtractorBackendResult, AgetError> {
    let tmp_dir = request
        .state_path
        .parent()
        .ok_or_else(|| AgetError::Stable {
            code: ErrorCode::IoError,
            message: format!(
                "owned extractor state path '{}' has no temp directory",
                request.state_path.display()
            ),
        })?;
    let extraction = extract_owned_static_or_rendered(
        tmp_dir,
        request.url,
        request.state,
        request.options,
        request.timeout,
        None,
    )?;

    let backend_response = ExtractorBackendResult {
        ok: true,
        final_url: Some(extraction.final_url),
        content: Some(extraction.content.clone()),
        warnings: extraction.warnings,
        error: None,
    };
    write_private_file(request.content_path, extraction.content.as_bytes())
        .map_err(io_aget_error)?;
    let metadata =
        serde_json::to_vec_pretty(&backend_response).map_err(|error| AgetError::Stable {
            code: ErrorCode::IoError,
            message: error.to_string(),
        })?;
    write_private_file(request.metadata_path, &metadata).map_err(io_aget_error)?;
    Ok(backend_response)
}

pub(crate) fn run_owned_browser_fallback(
    request: BrowserFallbackRequest<'_>,
) -> Result<BrowserFallbackResult, AgetError> {
    let extraction = extract_owned_static_or_rendered(
        request.tmp_dir,
        request.url,
        request.state,
        request.options,
        request.timeout,
        Some("body"),
    )?;
    let mut warnings = vec![OWNED_FALLBACK_WARNING.to_string()];
    warnings.extend(extraction.warnings);
    Ok(BrowserFallbackResult {
        final_url: extraction.final_url,
        content: extraction.content,
        warnings,
        extractor: OWNED_BROWSER_FALLBACK.to_string(),
    })
}

fn extract_owned_static_or_rendered(
    tmp_dir: &Path,
    url: &str,
    state: &PlaywrightState,
    options: &GetOptions,
    timeout: Duration,
    fallback_selector: Option<&str>,
) -> Result<OwnedPageExtraction, AgetError> {
    let owned_options = validate_owned_extraction_options(options)?;
    if owned_options.wait_for_images {
        return extract_owned_rendered_page(
            tmp_dir,
            url,
            state,
            options,
            timeout,
            fallback_selector,
            &owned_options,
        );
    }
    if !state.origins.is_empty() {
        return extract_owned_rendered_page(
            tmp_dir,
            url,
            state,
            options,
            timeout,
            fallback_selector,
            &owned_options,
        );
    }

    let response = owned_fetch(url, state, timeout)?;
    if should_render_scripted_response(&response.body) {
        return extract_owned_rendered_page(
            tmp_dir,
            url,
            state,
            options,
            timeout,
            fallback_selector,
            &owned_options,
        );
    }

    match extract_owned_page_response(response, options, fallback_selector, &owned_options) {
        Ok(extraction) => Ok(extraction),
        Err(error) if should_retry_with_rendered_wait(&error, options) => {
            extract_owned_rendered_page(
                tmp_dir,
                url,
                state,
                options,
                timeout,
                fallback_selector,
                &owned_options,
            )
        }
        Err(error) => Err(error),
    }
}

fn extract_owned_rendered_page(
    tmp_dir: &Path,
    url: &str,
    state: &PlaywrightState,
    options: &GetOptions,
    timeout: Duration,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    let rendered = crate::browser_cdp::render_page(BrowserRenderRequest {
        tmp_dir,
        url,
        state,
        wait_for_selector: options.wait_for_selector.as_deref(),
        wait_until: owned_options.wait_until,
        wait_for_images: owned_options.wait_for_images,
        flatten_shadow_dom: owned_options.flatten_shadow_dom,
        settle_delay: owned_options.render_settle_delay,
        page_timeout: owned_options.page_timeout.unwrap_or(timeout),
        wait_for_timeout: owned_options.wait_for_timeout,
        timeout,
    })?;
    let mut extraction = extract_owned_html(
        rendered.final_url,
        rendered.html,
        options,
        fallback_selector,
        owned_options,
    )?;
    extraction.warnings.extend(rendered.warnings);
    Ok(extraction)
}

fn should_retry_with_rendered_wait(error: &AgetError, options: &GetOptions) -> bool {
    if options.wait_for_selector.is_none() {
        return false;
    }
    matches!(
        error,
        AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message,
        } if message.starts_with("wait selector ") && message.ends_with(" was not found by owned extractor")
    )
}

struct OwnedPageExtraction {
    final_url: String,
    content: String,
    warnings: Vec<String>,
}

#[derive(Debug)]
struct OwnedExtractorOptions {
    excluded_tags: Vec<String>,
    target_elements: Vec<String>,
    only_text: bool,
    wait_until: PageWaitUntil,
    wait_for_images: bool,
    flatten_shadow_dom: bool,
    render_settle_delay: Duration,
    page_timeout: Option<Duration>,
    wait_for_timeout: Option<Duration>,
}

impl Default for OwnedExtractorOptions {
    fn default() -> Self {
        Self {
            excluded_tags: Vec::new(),
            target_elements: Vec::new(),
            only_text: false,
            wait_until: PageWaitUntil::Load,
            wait_for_images: false,
            flatten_shadow_dom: false,
            render_settle_delay: DEFAULT_RENDER_SETTLE_DELAY,
            page_timeout: None,
            wait_for_timeout: None,
        }
    }
}

fn extract_owned_page_response(
    response: OwnedHttpResponse,
    options: &GetOptions,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    extract_owned_html(
        response.final_url,
        response.body,
        options,
        fallback_selector,
        owned_options,
    )
}

fn should_render_scripted_response(body: &str) -> bool {
    let document = Html::parse_document(body);
    let Ok(selector) = parse_css_selector(
        "script[src],script:not([type]),script[type=\"text/javascript\"],script[type=\"module\"]",
    ) else {
        return false;
    };
    document.select(&selector).next().is_some()
}

fn extract_owned_html(
    final_url: String,
    body: String,
    options: &GetOptions,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    let mut document = Html::parse_document(&body);
    document = remove_selected_elements(document, "script,style,link,meta,noscript")?;

    if let Some(wait_for) = &options.wait_for_selector {
        let selector = parse_css_selector(wait_for)?;
        if document.select(&selector).next().is_none() {
            return Err(extraction_failed(format!(
                "wait selector '{wait_for}' was not found by owned extractor"
            )));
        }
    }

    document = remove_owned_overlay_elements(document)?;

    if !owned_options.excluded_tags.is_empty() {
        document = remove_owned_excluded_tags(document, &owned_options.excluded_tags)?;
    }

    if let Some(exclude_selector) = &options.exclude_selector {
        document = remove_selected_elements(document, exclude_selector)?;
    }

    let selector = options.selector.as_deref().or(fallback_selector);
    let base_url = markdown_base_url(&document, &final_url)?;
    let prefer_main_content = selector.is_none()
        && options.wait_for_selector.is_none()
        && options.content_format != OutputFormat::Html;
    let extracted = extract_owned_content(
        document,
        selector,
        &base_url,
        prefer_main_content,
        owned_options,
    )?;
    let content = match options.content_format {
        OutputFormat::Html => extracted.html,
        OutputFormat::Json => serde_json::json!({
            "url": final_url,
            "content": extracted.text,
        })
        .to_string(),
        OutputFormat::Markdown => extracted.markdown,
        OutputFormat::Text => extracted.text,
    };

    Ok(OwnedPageExtraction {
        final_url,
        content,
        warnings: Vec::new(),
    })
}

fn validate_owned_extraction_options(
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
            "word_count_threshold" => {
                // Crawl4AI's default cleaned-content path accepts this config but
                // currently hardcodes empty-leaf pruning to threshold 1.
                parse_owned_integer("crawl4ai.word_count_threshold", &option.value)?;
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
            "flatten_shadow_dom" => {
                owned_options.flatten_shadow_dom =
                    parse_owned_bool("crawl4ai.flatten_shadow_dom", &option.value)?;
            }
            _ => {
                return Err(extraction_failed(format!(
                    "owned extractor does not support backend option '{}'; supported options: crawl4ai.delay_before_return_html, crawl4ai.excluded_tags, crawl4ai.flatten_shadow_dom, crawl4ai.only_text, crawl4ai.page_timeout, crawl4ai.target_elements, crawl4ai.wait_for_images, crawl4ai.wait_for_timeout, crawl4ai.wait_until, crawl4ai.word_count_threshold",
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
    let seconds = value.trim().parse::<f64>().map_err(|_| {
        extraction_failed(format!(
            "crawl4ai.delay_before_return_html expects a non-negative number of seconds, got '{value}'"
        ))
    })?;
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(extraction_failed(format!(
            "crawl4ai.delay_before_return_html expects a non-negative finite number of seconds, got '{value}'"
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

fn parse_owned_integer(name: &str, value: &str) -> Result<i64, AgetError> {
    value
        .trim()
        .parse::<i64>()
        .map_err(|_| extraction_failed(format!("{name} expects an integer value, got '{value}'")))
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

struct ExtractedOwnedContent {
    html: String,
    markdown: String,
    text: String,
}

fn extract_owned_content(
    mut document: Html,
    selector: Option<&str>,
    base_url: &str,
    prefer_main_content: bool,
    owned_options: &OwnedExtractorOptions,
) -> Result<ExtractedOwnedContent, AgetError> {
    let root_ids = if let Some(raw_selector) = selector {
        match parse_css_selector(raw_selector) {
            Ok(selector) => {
                let selected = document
                    .select(&selector)
                    .map(|element| element.id())
                    .collect::<Vec<_>>();
                if selected.is_empty() {
                    vec![document.root_element().id()]
                } else {
                    selected
                }
            }
            Err(_) => vec![document.root_element().id()],
        }
    } else if !owned_options.target_elements.is_empty() {
        vec![document.root_element().id()]
    } else if prefer_main_content {
        vec![default_main_content_element(&document)?.id()]
    } else {
        vec![document.root_element().id()]
    };
    let target_ids = if owned_options.target_elements.is_empty() {
        Vec::new()
    } else {
        collect_target_owned_element_ids(&document, &root_ids, &owned_options.target_elements)?
    };

    // Match Crawl4AI's cleanup order: selectors see original attributes, but
    // serialized cleaned HTML keeps only its small important-attribute allowlist.
    document = clean_owned_base64_image_sources(document);
    document = remove_owned_empty_elements(document, &root_ids, &target_ids);
    document = prune_owned_unwanted_attributes(document);

    if owned_options.target_elements.is_empty() {
        if let [root_id] = root_ids.as_slice() {
            let root = element_by_id(&document, *root_id)?;
            return Ok(extract_single_owned_element(root, base_url, owned_options));
        }
        return extract_target_owned_elements(&document, &root_ids, base_url, owned_options);
    }
    extract_target_owned_elements(&document, &target_ids, base_url, owned_options)
}

fn extract_single_owned_element(
    element: ElementRef<'_>,
    base_url: &str,
    owned_options: &OwnedExtractorOptions,
) -> ExtractedOwnedContent {
    ExtractedOwnedContent {
        html: element.inner_html(),
        markdown: element_to_markdown(element, base_url, owned_options.only_text),
        text: normalize_text_pieces(element.text()),
    }
}

fn collect_target_owned_element_ids(
    document: &Html,
    source_ids: &[NodeId],
    raw_selectors: &[String],
) -> Result<Vec<NodeId>, AgetError> {
    let mut ids = Vec::new();
    for source_id in source_ids {
        let source = element_by_id(document, *source_id)?;
        for raw_selector in raw_selectors {
            let selector = parse_css_selector(raw_selector)?;
            ids.extend(source.select(&selector).map(|element| element.id()));
        }
    }
    Ok(ids)
}

fn extract_target_owned_elements(
    document: &Html,
    element_ids: &[NodeId],
    base_url: &str,
    owned_options: &OwnedExtractorOptions,
) -> Result<ExtractedOwnedContent, AgetError> {
    let elements = element_ids
        .iter()
        .map(|id| element_by_id(document, *id))
        .collect::<Result<Vec<_>, _>>()?;

    let html = elements
        .iter()
        .map(|element| element.html())
        .collect::<Vec<_>>()
        .join("\n");
    let markdown = normalize_markdown(
        &elements
            .iter()
            .map(|element| element_to_markdown(*element, base_url, owned_options.only_text))
            .filter(|markdown| !markdown.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
    );
    let text = normalize_text_pieces(elements.iter().flat_map(|element| element.text()));

    Ok(ExtractedOwnedContent {
        html,
        markdown,
        text,
    })
}

fn default_main_content_element(document: &Html) -> Result<ElementRef<'_>, AgetError> {
    if let Some(element) = best_main_content_candidate(document)? {
        return Ok(element);
    }
    if let Some(body) = first_selected_element(document, "body")? {
        return Ok(body);
    }
    Ok(document.root_element())
}

fn best_main_content_candidate<'a>(
    document: &'a Html,
) -> Result<Option<ElementRef<'a>>, AgetError> {
    let mut best = None;
    for selector in ["main", r#"[role="main"]"#, "article"] {
        let selector = parse_css_selector(selector)?;
        for element in document.select(&selector) {
            let score = score_main_content_candidate(element)?;
            if score <= 0 {
                continue;
            }
            let replace = best
                .as_ref()
                .map(|(best_score, _)| score > *best_score)
                .unwrap_or(true);
            if replace {
                best = Some((score, element.id()));
            }
        }
    }
    best.map(|(_, id)| element_by_id(document, id)).transpose()
}

fn score_main_content_candidate(element: ElementRef<'_>) -> Result<i64, AgetError> {
    let text_words = word_count(element.text());
    if text_words == 0 {
        return Ok(0);
    }
    let link_selector = parse_css_selector("a")?;
    let link_words = element
        .select(&link_selector)
        .map(|link| word_count(link.text()))
        .sum::<usize>();
    let tag_bonus = match element.value().name() {
        "main" => 300,
        "article" => 250,
        _ => 150,
    };
    let label_penalty = content_label_penalty(element) as i64;
    Ok((text_words as i64 * 10) - (link_words as i64 * 8) + tag_bonus - label_penalty)
}

fn word_count<'a>(pieces: impl IntoIterator<Item = &'a str>) -> usize {
    pieces
        .into_iter()
        .flat_map(str::split_whitespace)
        .filter(|word| !word.is_empty())
        .count()
}

fn content_label_penalty(element: ElementRef<'_>) -> usize {
    let label = ["id", "class", "role", "aria-label"]
        .into_iter()
        .filter_map(|attribute| element.attr(attribute))
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    [
        "nav", "footer", "header", "sidebar", "aside", "ad", "advert", "promo", "comment",
        "related", "share", "social",
    ]
    .into_iter()
    .filter(|needle| label.contains(needle))
    .count()
        * 200
}

fn first_selected_element<'a>(
    document: &'a Html,
    raw_selector: &str,
) -> Result<Option<ElementRef<'a>>, AgetError> {
    let selector = parse_css_selector(raw_selector)?;
    Ok(document.select(&selector).next())
}

fn element_by_id(document: &Html, id: NodeId) -> Result<ElementRef<'_>, AgetError> {
    document
        .tree
        .get(id)
        .and_then(ElementRef::wrap)
        .ok_or_else(|| extraction_failed("owned extractor lost a selected HTML element"))
}

fn markdown_base_url(document: &Html, final_url: &str) -> Result<String, AgetError> {
    let selector = parse_css_selector("base[href]")?;
    let Some(base_href) = document
        .select(&selector)
        .next()
        .and_then(|element| element.attr("href"))
    else {
        return Ok(final_url.to_string());
    };
    Ok(resolve_markdown_url(final_url, base_href))
}

fn normalize_text_pieces<'a>(pieces: impl IntoIterator<Item = &'a str>) -> String {
    pieces
        .into_iter()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}
