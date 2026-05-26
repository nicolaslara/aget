use crate::aget_browser::{AgetBrowser, CurrentTabRequest};
use crate::error::AgetError;
use crate::extraction::{BrowserFallbackBackend, BrowserFallbackRequest, BrowserFallbackResult};
use crate::session::{
    ChromeImportOptions, LoginCancelOptions, LoginCancelResult, LoginFinishOptions,
    LoginFinishResult, LoginStartOptions, LoginStartResult, Session,
};

use super::{
    BrowserAutomationBackend, BrowserCurrentTabBackend, BrowserCurrentTabRequest,
    BrowserCurrentTabResult,
};

#[derive(Clone, Debug, Default)]
pub struct AgetBrowserBackend {
    browser: AgetBrowser,
}

impl AgetBrowserBackend {
    pub fn new(browser: AgetBrowser) -> Self {
        Self { browser }
    }
}

impl BrowserAutomationBackend for AgetBrowserBackend {
    fn import_chrome(&self, options: ChromeImportOptions) -> Result<Session, AgetError> {
        self.browser.import_chrome_session(options)
    }

    fn start_login(&self, options: LoginStartOptions) -> Result<LoginStartResult, AgetError> {
        self.browser.start_login_session(options)
    }

    fn finish_login(&self, options: LoginFinishOptions) -> Result<LoginFinishResult, AgetError> {
        self.browser.finish_login_session(options)
    }

    fn cancel_login(&self, options: LoginCancelOptions) -> Result<LoginCancelResult, AgetError> {
        self.browser.cancel_login_session(options)
    }
}

impl BrowserFallbackBackend for AgetBrowserBackend {
    fn extract_with_state(
        &self,
        request: BrowserFallbackRequest<'_>,
    ) -> Result<BrowserFallbackResult, AgetError> {
        self.browser.extract_with_state(request)
    }
}

impl BrowserCurrentTabBackend for AgetBrowserBackend {
    fn render_current_tab(
        &self,
        request: BrowserCurrentTabRequest,
    ) -> Result<BrowserCurrentTabResult, AgetError> {
        let rendered = self.browser.render_current_tab(CurrentTabRequest {
            port: request.port,
            locale: request.locale.as_deref(),
            timezone_id: request.timezone_id.as_deref(),
            wait_for_selector: request.wait_for_selector.as_deref(),
            wait_for_images: request.wait_for_images,
            scan_full_page: request.scan_full_page,
            scroll_delay: request.scroll_delay,
            max_scroll_steps: request.max_scroll_steps,
            flatten_shadow_dom: request.flatten_shadow_dom,
            process_iframes: request.process_iframes,
            settle_delay: request.settle_delay,
            discovery_timeout: request.discovery_timeout,
            page_timeout: request.page_timeout,
            wait_for_timeout: request.wait_for_timeout,
            timeout: request.timeout,
            capture_screenshot: request.capture_screenshot,
        })?;
        Ok(BrowserCurrentTabResult {
            cdp_ws_url: rendered.cdp_ws_url,
            final_url: rendered.final_url,
            html: rendered.html,
            warnings: rendered.warnings,
            screenshot_png: rendered.screenshot_png,
        })
    }
}
