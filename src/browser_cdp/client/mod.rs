use std::net::TcpStream;
use std::time::{Duration, Instant};

use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

pub(super) use super::session_data::{
    cdp_cookies, frame_storage_candidate_origins, origin_storage_from_runtime_result,
    playwright_cookies_from_cdp, preferred_page_target_id, storage_candidate_origins,
};

mod navigation;
mod page;
mod runtime;
mod state;
mod transport;
pub(super) use page::{PageSession, ScreenshotClip};
pub(super) use runtime::fail_on_runtime_evaluation_exception;
#[cfg(test)]
pub(in crate::browser_cdp) use transport::cdp_websocket_config;

pub(super) struct CdpClient {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    next_id: u64,
    direct_page_connection: bool,
    keepalive_interval: Duration,
    last_keepalive: Instant,
}
