use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use serde_json::Value;

use super::{navigation_wait_timeout, network_request_id, CdpClient, NavigationOutcome};
use crate::error::AgetError;

const NETWORK_IDLE_DURATION: Duration = Duration::from_millis(500);

impl CdpClient {
    pub(super) fn wait_for_network_idle(
        &mut self,
        navigate_id: u64,
        session_id: &str,
        timeout: Duration,
    ) -> Result<(), AgetError> {
        let deadline = Instant::now() + timeout;
        let mut navigate_response_seen = false;
        let mut readiness_event_seen = false;
        let mut inflight_requests = BTreeSet::new();
        let mut idle_since = Instant::now();

        loop {
            if navigate_response_seen
                && readiness_event_seen
                && inflight_requests.is_empty()
                && idle_since.elapsed() >= NETWORK_IDLE_DURATION
            {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(navigation_wait_timeout("networkidle"));
            }

            let Some(message) = self.try_read_message(deadline)? else {
                continue;
            };

            if message.get("id").and_then(Value::as_u64) == Some(navigate_id) {
                if self.handle_navigate_response(&message)? == NavigationOutcome::SameDocument {
                    return Ok(());
                }
                navigate_response_seen = true;
                if inflight_requests.is_empty() {
                    idle_since = Instant::now();
                }
                continue;
            }
            if !self.message_matches_session(&message, session_id) {
                continue;
            }

            match message.get("method").and_then(Value::as_str) {
                Some("Page.domContentEventFired" | "Page.loadEventFired") => {
                    readiness_event_seen = true;
                    if inflight_requests.is_empty() {
                        idle_since = Instant::now();
                    }
                }
                Some("Network.requestWillBeSent") => {
                    if let Some(request_id) = network_request_id(&message) {
                        inflight_requests.insert(request_id.to_string());
                        idle_since = Instant::now();
                    }
                }
                Some("Network.loadingFinished" | "Network.loadingFailed") => {
                    if let Some(request_id) = network_request_id(&message) {
                        inflight_requests.remove(request_id);
                        if inflight_requests.is_empty() {
                            idle_since = Instant::now();
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
