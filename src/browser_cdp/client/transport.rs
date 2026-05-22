use std::io;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};

use super::CdpClient;
use crate::browser_cdp::io_aget_error;
use crate::error::{AgetError, ErrorCode};

const CDP_READ_POLL: Duration = Duration::from_millis(100);

impl CdpClient {
    pub(in crate::browser_cdp) fn connect(
        ws_url: &str,
        timeout: Duration,
    ) -> Result<Self, AgetError> {
        let (mut socket, _) = connect(ws_url).map_err(|error| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!("owned browser fallback could not connect to Chrome CDP: {error}"),
        })?;
        configure_socket_timeout(&mut socket, timeout)?;
        Ok(Self {
            socket,
            next_id: 1,
            direct_page_connection: is_direct_page_ws_url(ws_url),
        })
    }

    pub(in crate::browser_cdp) fn send(
        &mut self,
        method: &str,
        params: Option<Value>,
        session_id: Option<&str>,
        timeout: Duration,
    ) -> Result<Value, AgetError> {
        let id = self.next_id;
        self.next_id += 1;
        let mut command = serde_json::Map::new();
        command.insert("id".to_string(), json!(id));
        command.insert("method".to_string(), json!(method));
        if let Some(params) = params {
            command.insert("params".to_string(), params);
        }
        if let Some(session_id) = session_id {
            command.insert("sessionId".to_string(), json!(session_id));
        }
        let text = serde_json::to_string(&Value::Object(command)).map_err(io_aget_error)?;
        self.socket
            .send(Message::Text(text.into()))
            .map_err(cdp_io_error)?;
        self.wait_for_response(id, method, timeout)
    }

    pub(super) fn send_no_wait(
        &mut self,
        method: &str,
        params: Option<Value>,
        session_id: Option<&str>,
    ) -> Result<u64, AgetError> {
        let id = self.next_id;
        self.next_id += 1;
        let mut command = serde_json::Map::new();
        command.insert("id".to_string(), json!(id));
        command.insert("method".to_string(), json!(method));
        if let Some(params) = params {
            command.insert("params".to_string(), params);
        }
        if let Some(session_id) = session_id {
            command.insert("sessionId".to_string(), json!(session_id));
        }
        let text = serde_json::to_string(&Value::Object(command)).map_err(io_aget_error)?;
        self.socket
            .send(Message::Text(text.into()))
            .map_err(cdp_io_error)?;
        Ok(id)
    }

    fn wait_for_response(
        &mut self,
        id: u64,
        method: &str,
        timeout: Duration,
    ) -> Result<Value, AgetError> {
        let deadline = Instant::now() + timeout;
        loop {
            let message = self.read_message(deadline)?;
            if message.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(error) = message.get("error") {
                let text = error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("CDP command failed");
                return Err(AgetError::Stable {
                    code: ErrorCode::ExtractionFailed,
                    message: format!("owned browser fallback CDP {method} failed: {text}"),
                });
            }
            return Ok(message.get("result").cloned().unwrap_or(Value::Null));
        }
    }

    pub(super) fn read_message(&mut self, deadline: Instant) -> Result<Value, AgetError> {
        loop {
            if let Some(message) = self.try_read_message(deadline)? {
                return Ok(message);
            }
        }
    }

    pub(super) fn try_read_message(
        &mut self,
        deadline: Instant,
    ) -> Result<Option<Value>, AgetError> {
        if Instant::now() >= deadline {
            return Err(AgetError::Stable {
                code: ErrorCode::Timeout,
                message: "owned browser fallback timed out waiting for Chrome CDP".to_string(),
            });
        }
        match self.socket.read() {
            Ok(Message::Text(text)) => {
                let text: &str = text.as_ref();
                Ok(serde_json::from_str(text).ok())
            }
            Ok(Message::Binary(bytes)) => {
                let Ok(text) = String::from_utf8(bytes.to_vec()) else {
                    return Ok(None);
                };
                Ok(serde_json::from_str(&text).ok())
            }
            Ok(Message::Ping(bytes)) => {
                self.socket
                    .send(Message::Pong(bytes))
                    .map_err(cdp_io_error)?;
                Ok(None)
            }
            Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => Ok(None),
            Ok(Message::Close(_)) => Err(AgetError::Stable {
                code: ErrorCode::ExtractionFailed,
                message: "owned browser fallback Chrome CDP connection closed".to_string(),
            }),
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                Ok(None)
            }
            Err(error) => Err(cdp_io_error(error)),
        }
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

pub(super) fn remaining(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now())
}

fn cdp_io_error(error: tungstenite::Error) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::ExtractionFailed,
        message: format!("owned browser fallback Chrome CDP I/O failed: {error}"),
    }
}

fn is_direct_page_ws_url(ws_url: &str) -> bool {
    ws_url.contains("/devtools/page/") || ws_url.contains("/devtools/webview/")
}
