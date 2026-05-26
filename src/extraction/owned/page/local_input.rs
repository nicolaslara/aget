use std::fs;
use std::path::Path;

use url::Url;

use crate::error::AgetError;

use super::super::super::{extraction_failed, GetOptions};
use super::super::options::OwnedExtractorOptions;

pub(super) struct LocalBrowserRenderInput {
    pub(super) render_url: String,
    pub(super) final_url: String,
}

pub(super) fn should_route_local_input_through_browser(
    options: &GetOptions,
    owned_options: &OwnedExtractorOptions,
) -> bool {
    owned_options.process_in_browser
        || owned_options.wait_for_images
        || owned_options.process_iframes
        || owned_options.scan_full_page
        || browser_context_options_require_render(owned_options)
        || options.wait_for_selector.is_some()
}

pub(super) fn browser_context_options_require_render(
    owned_options: &OwnedExtractorOptions,
) -> bool {
    owned_options.locale.is_some() || owned_options.timezone_id.is_some()
}

pub(super) fn local_browser_render_input(
    tmp_dir: &Path,
    url: &str,
) -> Result<Option<LocalBrowserRenderInput>, AgetError> {
    if let Some(path) = url.strip_prefix("file://") {
        let render_url = Url::from_file_path(path).map_err(|_| {
            extraction_failed(format!(
                "could not convert local file path '{path}' to file URL"
            ))
        })?;
        return Ok(Some(LocalBrowserRenderInput {
            render_url: render_url.to_string(),
            final_url: url.to_string(),
        }));
    }
    let html = if let Some(html) = url.strip_prefix("raw://") {
        html
    } else if let Some(html) = url.strip_prefix("raw:") {
        html
    } else {
        return Ok(None);
    };
    let path = tmp_dir.join("raw-browser-input.html");
    fs::write(&path, html)
        .map_err(|error| extraction_failed(format!("write raw browser input: {error}")))?;
    let render_url = Url::from_file_path(&path).map_err(|_| {
        extraction_failed(format!(
            "could not convert raw browser input path '{}' to file URL",
            path.display()
        ))
    })?;
    Ok(Some(LocalBrowserRenderInput {
        render_url: render_url.to_string(),
        final_url: url.to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use crate::cli::OutputFormat;
    use crate::extraction::GetOptions;

    use super::super::super::options::OwnedExtractorOptions;
    use super::{
        browser_context_options_require_render, local_browser_render_input,
        should_route_local_input_through_browser,
    };

    #[test]
    fn local_browser_render_input_writes_raw_html_to_temp_file_url() {
        let temp = tempfile::tempdir().unwrap();
        let input =
            local_browser_render_input(temp.path(), "raw:<html><body>Raw</body></html>").unwrap();
        let input = input.unwrap();
        assert_eq!(input.final_url, "raw:<html><body>Raw</body></html>");
        assert!(input.render_url.starts_with("file://"));
        assert_eq!(
            std::fs::read_to_string(temp.path().join("raw-browser-input.html")).unwrap(),
            "<html><body>Raw</body></html>"
        );
    }

    #[test]
    fn local_browser_routing_stays_off_without_browser_options() {
        let options = GetOptions {
            url: "raw:<main>Local</main>".to_string(),
            sessions: Vec::new(),
            output: None,
            home: None,
            timeout: None,
            content_format: OutputFormat::Markdown,
            selector: None,
            exclude_selector: None,
            wait_for_selector: None,
            max_chars: None,
            cache_policy: crate::cli::CachePolicy::Off,
            cache_ttl: std::time::Duration::from_secs(0),
            backend_options: Vec::new(),
        };
        assert!(!should_route_local_input_through_browser(
            &options,
            &OwnedExtractorOptions::default()
        ));
        let mut owned_options = OwnedExtractorOptions::default();
        owned_options.process_in_browser = true;
        assert!(should_route_local_input_through_browser(
            &options,
            &owned_options
        ));
    }

    #[test]
    fn browser_context_options_require_browser_rendering() {
        let options = GetOptions {
            url: "https://example.com/page".to_string(),
            sessions: Vec::new(),
            output: None,
            home: None,
            timeout: None,
            content_format: OutputFormat::Markdown,
            selector: None,
            exclude_selector: None,
            wait_for_selector: None,
            max_chars: None,
            cache_policy: crate::cli::CachePolicy::Off,
            cache_ttl: std::time::Duration::from_secs(0),
            backend_options: Vec::new(),
        };

        let mut owned_options = OwnedExtractorOptions {
            locale: Some("sv-SE".to_string()),
            ..OwnedExtractorOptions::default()
        };
        assert!(browser_context_options_require_render(&owned_options));
        assert!(should_route_local_input_through_browser(
            &options,
            &owned_options
        ));

        owned_options.locale = None;
        owned_options.timezone_id = Some("Europe/Stockholm".to_string());
        assert!(browser_context_options_require_render(&owned_options));
        assert!(should_route_local_input_through_browser(
            &options,
            &owned_options
        ));
    }
}
