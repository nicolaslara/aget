mod config;
mod dialogs;
mod frames;
mod keepalive;

use super::super::{read_cdp_request, reply_ok, reply_ok_binary};

fn mock_browser_ws_url(port: u16) -> String {
    format!("ws://127.0.0.1:{port}/devtools/browser/mock")
}
