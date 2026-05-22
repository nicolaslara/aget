use std::time::Duration;

use crate::browser_cdp::{
    discover_cdp_ws_url, render_attached_page, BrowserAttachedPageRenderRequest,
};
use crate::error::AgetError;

use super::AgetBrowser;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct CdpEndpointRequest {
    pub(crate) port: u16,
    pub(crate) timeout: Duration,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct CdpEndpointResult {
    pub(crate) ws_url: String,
}

#[derive(Debug, Clone)]
// Current-tab wiring intentionally stops at the engine seam until public
// consent and endpoint-discovery UX are chosen.
#[allow(dead_code)]
pub(crate) struct AttachedPageRequest<'a> {
    pub(crate) ws_url: &'a str,
    pub(crate) wait_for_selector: Option<&'a str>,
    pub(crate) wait_for_images: bool,
    pub(crate) scan_full_page: bool,
    pub(crate) scroll_delay: Duration,
    pub(crate) max_scroll_steps: usize,
    pub(crate) flatten_shadow_dom: bool,
    pub(crate) process_iframes: bool,
    pub(crate) settle_delay: Duration,
    pub(crate) page_timeout: Duration,
    pub(crate) wait_for_timeout: Option<Duration>,
    pub(crate) timeout: Duration,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct AttachedPageResult {
    pub(crate) final_url: String,
    pub(crate) html: String,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Clone)]
// Current-tab callers must make the consent/port choice before reaching this
// seam; this method only composes local CDP discovery with page capture.
#[allow(dead_code)]
pub(crate) struct CurrentTabRequest<'a> {
    pub(crate) port: u16,
    pub(crate) wait_for_selector: Option<&'a str>,
    pub(crate) wait_for_images: bool,
    pub(crate) scan_full_page: bool,
    pub(crate) scroll_delay: Duration,
    pub(crate) max_scroll_steps: usize,
    pub(crate) flatten_shadow_dom: bool,
    pub(crate) process_iframes: bool,
    pub(crate) settle_delay: Duration,
    pub(crate) discovery_timeout: Duration,
    pub(crate) page_timeout: Duration,
    pub(crate) wait_for_timeout: Option<Duration>,
    pub(crate) timeout: Duration,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct CurrentTabResult {
    pub(crate) cdp_ws_url: String,
    pub(crate) final_url: String,
    pub(crate) html: String,
    pub(crate) warnings: Vec<String>,
}

impl AgetBrowser {
    #[allow(dead_code)]
    pub(crate) fn discover_cdp_endpoint(
        &self,
        request: CdpEndpointRequest,
    ) -> Result<CdpEndpointResult, AgetError> {
        let ws_url = discover_cdp_ws_url(request.port, request.timeout)?;
        Ok(CdpEndpointResult { ws_url })
    }

    #[allow(dead_code)]
    pub(crate) fn render_current_tab(
        &self,
        request: CurrentTabRequest<'_>,
    ) -> Result<CurrentTabResult, AgetError> {
        let endpoint = self.discover_cdp_endpoint(CdpEndpointRequest {
            port: request.port,
            timeout: request.discovery_timeout,
        })?;
        let rendered = self.render_attached_page(AttachedPageRequest {
            ws_url: &endpoint.ws_url,
            wait_for_selector: request.wait_for_selector,
            wait_for_images: request.wait_for_images,
            scan_full_page: request.scan_full_page,
            scroll_delay: request.scroll_delay,
            max_scroll_steps: request.max_scroll_steps,
            flatten_shadow_dom: request.flatten_shadow_dom,
            process_iframes: request.process_iframes,
            settle_delay: request.settle_delay,
            page_timeout: request.page_timeout,
            wait_for_timeout: request.wait_for_timeout,
            timeout: request.timeout,
        })?;
        Ok(CurrentTabResult {
            cdp_ws_url: endpoint.ws_url,
            final_url: rendered.final_url,
            html: rendered.html,
            warnings: rendered.warnings,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn render_attached_page(
        &self,
        request: AttachedPageRequest<'_>,
    ) -> Result<AttachedPageResult, AgetError> {
        let rendered = render_attached_page(BrowserAttachedPageRenderRequest {
            ws_url: request.ws_url,
            wait_for_selector: request.wait_for_selector,
            wait_for_images: request.wait_for_images,
            scan_full_page: request.scan_full_page,
            scroll_delay: request.scroll_delay,
            max_scroll_steps: request.max_scroll_steps,
            flatten_shadow_dom: request.flatten_shadow_dom,
            process_iframes: request.process_iframes,
            settle_delay: request.settle_delay,
            page_timeout: request.page_timeout,
            wait_for_timeout: request.wait_for_timeout,
            timeout: request.timeout,
        })?;
        Ok(AttachedPageResult {
            final_url: rendered.final_url,
            html: rendered.html,
            warnings: rendered.warnings,
        })
    }
}
