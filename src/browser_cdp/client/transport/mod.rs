mod connection;
mod message;

use std::time::{Duration, Instant};

use tungstenite::protocol::WebSocketConfig;

const CDP_READ_POLL: Duration = Duration::from_millis(100);
const CDP_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

pub(in crate::browser_cdp) fn cdp_websocket_config() -> WebSocketConfig {
    WebSocketConfig::default()
        .max_message_size(None)
        .max_frame_size(None)
}

pub(super) fn remaining(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now())
}

pub(super) fn cdp_io_error(error: tungstenite::Error) -> crate::error::AgetError {
    crate::error::AgetError::Stable {
        code: crate::error::ErrorCode::ExtractionFailed,
        message: format!("owned browser fallback Chrome CDP I/O failed: {error}"),
    }
}

fn is_direct_page_ws_url(ws_url: &str) -> bool {
    ws_url.contains("/devtools/page/") || ws_url.contains("/devtools/webview/")
}
