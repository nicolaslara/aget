use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::cli::{ExtractorOption, OutputFormat};
use crate::error::{AgetError, ErrorCode};
use crate::extraction::{
    extract_owned_rendered_html, finish_direct_extraction, validate_owned_extraction_options,
    GetOptions, GetSuccess,
};

use super::{
    AgetWith, BrowserCurrentTabBackend, BrowserCurrentTabRequest, SessionStoreBackend,
    DEFAULT_TIMEOUT,
};

const CURRENT_TAB_EXTRACTOR: &str = "aget-owned-current-tab";

#[derive(Clone, Debug)]
pub struct CurrentTabOptions {
    pub port: u16,
    pub allow_private_content: bool,
    pub output: Option<PathBuf>,
    pub timeout: Option<Duration>,
    pub content_format: OutputFormat,
    pub selector: Option<String>,
    pub exclude_selector: Option<String>,
    pub wait_for_selector: Option<String>,
    pub max_chars: Option<usize>,
    pub backend_options: Vec<ExtractorOption>,
}

impl CurrentTabOptions {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            allow_private_content: false,
            output: None,
            timeout: None,
            content_format: OutputFormat::Markdown,
            selector: None,
            exclude_selector: None,
            wait_for_selector: None,
            max_chars: None,
            backend_options: Vec::new(),
        }
    }
}

impl<E, S, B> AgetWith<E, S, B>
where
    S: SessionStoreBackend + Clone,
    B: BrowserCurrentTabBackend + Clone,
{
    pub fn current_tab(&self, mut options: CurrentTabOptions) -> Result<GetSuccess, AgetError> {
        if !options.allow_private_content {
            return Err(AgetError::Stable {
                code: ErrorCode::UsageError,
                message: "current-tab reads the selected browser tab and may include authenticated/private content; pass explicit consent before running".to_string(),
            });
        }

        let started = Instant::now();
        let timeout = options.timeout.or(self.timeout).unwrap_or(DEFAULT_TIMEOUT);
        let get_options = GetOptions {
            url: "current-tab".to_string(),
            sessions: Vec::new(),
            output: options.output.take(),
            home: Some(self.session_store.home().to_path_buf()),
            timeout: Some(timeout),
            content_format: options.content_format,
            selector: options.selector.take(),
            exclude_selector: options.exclude_selector.take(),
            wait_for_selector: options.wait_for_selector.take(),
            max_chars: options.max_chars,
            backend_options: options.backend_options,
        };
        let owned_options = validate_owned_extraction_options(&get_options)?;
        let rendered = self
            .browser_backend
            .render_current_tab(BrowserCurrentTabRequest {
                port: options.port,
                wait_for_selector: get_options.wait_for_selector.clone(),
                wait_for_images: owned_options.wait_for_images,
                scan_full_page: owned_options.scan_full_page,
                scroll_delay: owned_options.scroll_delay,
                max_scroll_steps: owned_options.max_scroll_steps,
                flatten_shadow_dom: owned_options.flatten_shadow_dom,
                process_iframes: owned_options.process_iframes,
                settle_delay: owned_options.render_settle_delay,
                discovery_timeout: timeout,
                page_timeout: owned_options.page_timeout.unwrap_or(timeout),
                wait_for_timeout: owned_options.wait_for_timeout,
                timeout,
            })?;
        let mut extraction =
            extract_owned_rendered_html(rendered.final_url, rendered.html, &get_options)?;
        let mut warnings =
            vec!["current-tab content may include authenticated/private browser data".to_string()];
        warnings.push(format!(
            "current-tab used local CDP endpoint {}",
            rendered.cdp_ws_url
        ));
        warnings.extend(rendered.warnings);
        warnings.append(&mut extraction.warnings);

        finish_direct_extraction(
            get_options,
            extraction.final_url,
            extraction.content,
            warnings,
            CURRENT_TAB_EXTRACTOR,
            true,
            started,
        )
    }
}
