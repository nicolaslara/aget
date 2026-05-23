use std::fs;
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use crate::browser_cdp::client::CdpClient;
use crate::error::{AgetError, ErrorCode};

use super::{discover_cdp_ws_url, read_devtools_active_port};

pub(in crate::browser_cdp) fn connect_existing_profile_browser(
    profile_dir: &Path,
    timeout: Duration,
) -> Result<Option<CdpClient>, AgetError> {
    let Some((port, path)) = read_devtools_active_port(profile_dir) else {
        return Ok(None);
    };
    let ws_url = format!("ws://127.0.0.1:{port}{path}");
    match CdpClient::connect(&ws_url, timeout) {
        Ok(client) => Ok(Some(client)),
        Err(AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            ..
        }) => {
            let discovered_ws_url = match discover_cdp_ws_url(port, timeout) {
                Ok(ws_url) => ws_url,
                Err(_) => {
                    remove_stale_devtools_active_port(profile_dir);
                    return Ok(None);
                }
            };
            match CdpClient::connect(&discovered_ws_url, timeout) {
                Ok(client) => Ok(Some(client)),
                Err(AgetError::Stable {
                    code: ErrorCode::BackendUnavailable,
                    ..
                }) => {
                    remove_stale_devtools_active_port(profile_dir);
                    Ok(None)
                }
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

fn remove_stale_devtools_active_port(profile_dir: &Path) {
    let _ = fs::remove_file(profile_dir.join("DevToolsActivePort"));
}

pub(in crate::browser_cdp) fn wait_for_profile_browser_shutdown(
    profile_dir: &Path,
    timeout: Duration,
) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let Some((port, _)) = read_devtools_active_port(profile_dir) else {
            return;
        };
        if TcpStream::connect(("127.0.0.1", port)).is_err() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
}
