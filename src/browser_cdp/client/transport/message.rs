use std::io;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tungstenite::Message;

use super::cdp_io_error;
use crate::browser_cdp::client::CdpClient;
use crate::browser_cdp::io_aget_error;
use crate::error::{AgetError, ErrorCode};

impl CdpClient {
    pub(in crate::browser_cdp) fn send(
        &mut self,
        method: &str,
        params: Option<Value>,
        session_id: Option<&str>,
        timeout: Duration,
    ) -> Result<Value, AgetError> {
        let id = self.send_no_wait(method, params, session_id)?;
        self.wait_for_response(id, method, timeout)
    }

    pub(in crate::browser_cdp::client) fn send_no_wait(
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

    pub(in crate::browser_cdp::client) fn read_message(
        &mut self,
        deadline: Instant,
    ) -> Result<Value, AgetError> {
        loop {
            if let Some(message) = self.try_read_message(deadline)? {
                return Ok(message);
            }
            self.send_keepalive_if_due()?;
        }
    }

    pub(in crate::browser_cdp::client) fn try_read_message(
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
                self.parse_cdp_message(text)
            }
            Ok(Message::Binary(bytes)) => {
                let Ok(text) = String::from_utf8(bytes.to_vec()) else {
                    return Ok(None);
                };
                self.parse_cdp_message(&text)
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

    fn parse_cdp_message(&mut self, text: &str) -> Result<Option<Value>, AgetError> {
        let Ok(message) = serde_json::from_str::<Value>(text) else {
            return Ok(None);
        };
        if self.auto_handle_dialog_if_needed(&message)? {
            return Ok(None);
        }
        Ok(Some(message))
    }

    fn auto_handle_dialog_if_needed(&mut self, message: &Value) -> Result<bool, AgetError> {
        if message.get("method").and_then(Value::as_str) != Some("Page.javascriptDialogOpening") {
            return Ok(false);
        }
        let dialog_type = message
            .get("params")
            .and_then(|params| params.get("type"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !matches!(dialog_type, "alert" | "beforeunload") {
            return Ok(false);
        }
        let session_id = message.get("sessionId").and_then(Value::as_str);
        self.send_no_wait(
            "Page.handleJavaScriptDialog",
            Some(json!({ "accept": true })),
            session_id,
        )?;
        Ok(true)
    }

    fn send_keepalive_if_due(&mut self) -> Result<(), AgetError> {
        let now = Instant::now();
        if now.duration_since(self.last_keepalive) < self.keepalive_interval {
            return Ok(());
        }
        self.socket
            .send(Message::Ping(Vec::<u8>::new().into()))
            .map_err(cdp_io_error)?;
        self.last_keepalive = now;
        Ok(())
    }

    #[cfg(test)]
    pub(in crate::browser_cdp) fn set_keepalive_interval_for_test(&mut self, interval: Duration) {
        self.keepalive_interval = interval;
        self.last_keepalive = Instant::now();
    }
}
