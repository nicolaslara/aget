mod close;
mod domains;
mod emulation;
mod preload;
mod targets;

pub(in crate::browser_cdp) struct PageSession {
    /// Empty for direct page WebSocket connections, where CDP commands already
    /// target the page and Chrome does not return a flattened target session.
    pub(in crate::browser_cdp) target_id: String,
    pub(in crate::browser_cdp) session_id: String,
}

impl super::CdpClient {
    pub(super) fn session_param<'a>(&self, session_id: &'a str) -> Option<&'a str> {
        if session_id.is_empty() {
            None
        } else {
            Some(session_id)
        }
    }
}
