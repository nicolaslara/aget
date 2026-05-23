use std::time::{Duration, Instant};

use serde_json::Value;

use super::{navigation_wait_timeout, CdpClient, NavigationOutcome};
use crate::error::AgetError;

impl CdpClient {
    pub(super) fn wait_for_navigation_event(
        &mut self,
        navigate_id: u64,
        event_name: &str,
        session_id: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() >= deadline {
                return Err(navigation_wait_timeout(event_name));
            }
            let Some(message) = self.try_read_message(deadline)? else {
                continue;
            };
            if message.get("id").and_then(Value::as_u64) == Some(navigate_id) {
                if self.handle_navigate_response(&message)? == NavigationOutcome::SameDocument {
                    return Ok(());
                }
                continue;
            }
            if message.get("method").and_then(Value::as_str) != Some(event_name) {
                continue;
            }
            if self.message_matches_session(&message, session_id) {
                return Ok(());
            }
        }
    }
}
