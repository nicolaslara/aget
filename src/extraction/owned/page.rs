use std::path::Path;
use std::time::Duration;

use scraper::Html;

use crate::browser_cdp::BrowserRenderRequest;
use crate::cli::OutputFormat;
use crate::error::{AgetError, ErrorCode};
use crate::session::PlaywrightState;

use super::super::html_clean::{
    parse_css_selector, remove_owned_comments, remove_owned_excluded_domain_urls,
    remove_owned_excluded_tags, remove_owned_external_images, remove_owned_external_links,
    remove_owned_internal_links, remove_owned_overlay_elements, remove_owned_social_media_links,
    remove_selected_elements,
};
use super::super::http::{owned_fetch, OwnedHttpResponse};
use super::super::{extraction_failed, GetOptions};
use super::content::{extract_owned_content, markdown_base_url};
use super::options::{validate_owned_extraction_options, OwnedExtractorOptions};

pub(crate) struct OwnedPageExtraction {
    pub(crate) final_url: String,
    pub(crate) content: String,
    pub(crate) warnings: Vec<String>,
}

pub(super) fn extract_owned_static_or_rendered(
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
        scan_full_page: owned_options.scan_full_page,
        scroll_delay: owned_options.scroll_delay,
        max_scroll_steps: owned_options.max_scroll_steps,
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

pub(crate) fn extract_owned_rendered_html(
    final_url: String,
    html: String,
    options: &GetOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    let owned_options = validate_owned_extraction_options(options)?;
    extract_owned_html(final_url, html, options, None, &owned_options)
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
    let Ok(selector) = parse_css_selector("script") else {
        return false;
    };
    document
        .select(&selector)
        .any(|script| is_executable_script_type(script.attr("type")))
}

fn is_executable_script_type(script_type: Option<&str>) -> bool {
    let Some(script_type) = script_type else {
        return true;
    };
    let script_type = script_type
        .split(';')
        .next()
        .unwrap_or(script_type)
        .trim()
        .to_ascii_lowercase();
    if script_type.is_empty() {
        return true;
    }
    matches!(
        script_type.as_str(),
        "module"
            | "text/javascript"
            | "application/javascript"
            | "text/ecmascript"
            | "application/ecmascript"
            | "text/jscript"
    )
}

fn extract_owned_html(
    final_url: String,
    body: String,
    options: &GetOptions,
    fallback_selector: Option<&str>,
    owned_options: &OwnedExtractorOptions,
) -> Result<OwnedPageExtraction, AgetError> {
    let mut document = Html::parse_document(&body);
    document = remove_owned_comments(document);
    document = remove_selected_elements(document, "script,style,link,meta,noscript")?;

    if let Some(wait_for) = &options.wait_for_selector {
        let selector = parse_css_selector(wait_for)?;
        if document.select(&selector).next().is_none() {
            return Err(extraction_failed(format!(
                "wait selector '{wait_for}' was not found by owned extractor"
            )));
        }
    }

    if owned_options.remove_overlay_elements {
        document = remove_owned_overlay_elements(document)?;
    }

    if !owned_options.excluded_tags.is_empty() {
        document = remove_owned_excluded_tags(document, &owned_options.excluded_tags)?;
    }

    if let Some(exclude_selector) = &options.exclude_selector {
        document = remove_selected_elements(document, exclude_selector)?;
    }

    if owned_options.remove_forms {
        document = remove_selected_elements(document, "form")?;
    }

    if owned_options.exclude_all_images {
        document = remove_selected_elements(document, "img")?;
    }

    let selector = options.selector.as_deref().or(fallback_selector);
    let base_url = markdown_base_url(&document, &final_url)?;

    if !owned_options.exclude_domains.is_empty() {
        document =
            remove_owned_excluded_domain_urls(document, &base_url, &owned_options.exclude_domains)?;
    }

    if owned_options.exclude_external_links {
        document = remove_owned_external_links(document, &base_url)?;
    }

    if owned_options.exclude_internal_links {
        document = remove_owned_internal_links(document, &base_url)?;
    }

    if owned_options.exclude_social_media_links {
        document = remove_owned_social_media_links(
            document,
            &base_url,
            &owned_options.exclude_social_media_domains,
        )?;
    }

    if owned_options.exclude_external_images {
        document = remove_owned_external_images(document, &base_url)?;
    }

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

#[cfg(test)]
mod tests {
    use super::should_render_scripted_response;

    #[test]
    fn script_detection_covers_common_executable_javascript_types() {
        for script in [
            r#"<script>window.ready = true</script>"#,
            r#"<script src="/app.js"></script>"#,
            r#"<script type="">window.ready = true</script>"#,
            r#"<script type="module">window.ready = true</script>"#,
            r#"<script type="text/javascript">window.ready = true</script>"#,
            r#"<script type="text/javascript; charset=utf-8">window.ready = true</script>"#,
            r#"<script type="application/javascript">window.ready = true</script>"#,
            r#"<script type="text/ecmascript">window.ready = true</script>"#,
            r#"<script type="application/ecmascript">window.ready = true</script>"#,
        ] {
            assert!(
                should_render_scripted_response(&format!("<html><body>{script}</body></html>")),
                "expected executable script detection for {script}"
            );
        }
    }

    #[test]
    fn script_detection_ignores_non_executable_data_script_types() {
        for script in [
            r#"<script type="application/ld+json">{"name":"Docs"}</script>"#,
            r#"<script type="application/json">{"name":"Docs"}</script>"#,
            r#"<script type="application/json" src="/data.json"></script>"#,
            r#"<script type="importmap">{"imports":{}}</script>"#,
            r#"<script type="speculationrules">{"prerender":[]}</script>"#,
        ] {
            assert!(
                !should_render_scripted_response(&format!("<html><body>{script}</body></html>")),
                "expected static extraction for {script}"
            );
        }
    }
}
