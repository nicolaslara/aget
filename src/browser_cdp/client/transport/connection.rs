use std::time::{Duration, Instant};

use tungstenite::client::connect_with_config;
use tungstenite::stream::MaybeTlsStream;

use super::{cdp_websocket_config, is_direct_page_ws_url, CDP_KEEPALIVE_INTERVAL, CDP_READ_POLL};
use crate::browser_cdp::client::CdpClient;
use crate::browser_cdp::io_aget_error;
use crate::error::{AgetError, ErrorCode};

impl CdpClient {
    pub(in crate::browser_cdp) fn connect(
        ws_url: &str,
        timeout: Duration,
    ) -> Result<Self, AgetError> {
        let (mut socket, _) = connect_with_config(ws_url, Some(cdp_websocket_config()), 3)
            .map_err(|error| AgetError::Stable {
                code: ErrorCode::BackendUnavailable,
                message: format!("owned browser fallback could not connect to Chrome CDP: {error}"),
            })?;
        configure_socket_timeout(&mut socket, timeout)?;
        Ok(Self {
            socket,
            next_id: 1,
            direct_page_connection: is_direct_page_ws_url(ws_url),
            keepalive_interval: CDP_KEEPALIVE_INTERVAL,
            last_keepalive: Instant::now(),
        })
    }
}

fn configure_socket_timeout(
    socket: &mut tungstenite::WebSocket<MaybeTlsStream<std::net::TcpStream>>,
    timeout: Duration,
) -> Result<(), AgetError> {
    let timeout = timeout.min(CDP_READ_POLL).max(Duration::from_millis(10));
    match socket.get_mut() {
        MaybeTlsStream::Plain(stream) => {
            stream
                .set_read_timeout(Some(timeout))
                .map_err(io_aget_error)?;
            stream
                .set_write_timeout(Some(timeout))
                .map_err(io_aget_error)?;
        }
        #[allow(unreachable_patterns)]
        _ => {}
    }
    Ok(())
}
