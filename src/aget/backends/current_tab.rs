use std::time::Duration;

use crate::error::AgetError;

pub struct BrowserCurrentTabRequest {
    pub port: u16,
    pub locale: Option<String>,
    pub timezone_id: Option<String>,
    pub wait_for_selector: Option<String>,
    pub wait_for_images: bool,
    pub scan_full_page: bool,
    pub scroll_delay: Duration,
    pub max_scroll_steps: usize,
    pub flatten_shadow_dom: bool,
    pub process_iframes: bool,
    pub settle_delay: Duration,
    pub discovery_timeout: Duration,
    pub page_timeout: Duration,
    pub wait_for_timeout: Option<Duration>,
    pub timeout: Duration,
}

pub struct BrowserCurrentTabResult {
    pub cdp_ws_url: String,
    pub final_url: String,
    pub html: String,
    pub warnings: Vec<String>,
}

pub trait BrowserCurrentTabBackend {
    fn render_current_tab(
        &self,
        request: BrowserCurrentTabRequest,
    ) -> Result<BrowserCurrentTabResult, AgetError>;
}
