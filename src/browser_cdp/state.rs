use std::path::Path;
use std::time::Duration;

use serde_json::json;

use crate::error::AgetError;
use crate::session::PlaywrightState;

use super::chrome_process::ChromeProcess;
use super::client::CdpClient;
use super::CHROME_SHUTDOWN_WAIT;

pub(crate) struct BrowserStateExportRequest<'a> {
    pub(crate) profile_dir: &'a Path,
    pub(crate) profile_directory: Option<&'a str>,
    pub(crate) use_real_keychain: bool,
    pub(crate) allowed_domains: &'a [String],
    pub(crate) timeout: Duration,
}

pub(crate) fn export_browser_state(
    request: BrowserStateExportRequest<'_>,
) -> Result<PlaywrightState, AgetError> {
    let mut chrome = ChromeProcess::launch_profile(
        request.profile_dir,
        request.profile_directory,
        request.use_real_keychain,
        request.timeout,
        "owned Chrome import",
    )?;
    let mut client = CdpClient::connect(&chrome.ws_url, request.timeout)?;
    let page = client.create_page(request.timeout)?;
    client.enable_page_domains(&page.session_id, request.timeout)?;
    let state = client.export_state(&page.session_id, request.allowed_domains, request.timeout)?;
    let _ = client.send(
        "Target.closeTarget",
        Some(json!({ "targetId": page.target_id })),
        None,
        Duration::from_secs(1),
    );
    let _ = client.send("Browser.close", None, None, Duration::from_secs(1));
    chrome.wait_or_kill(CHROME_SHUTDOWN_WAIT);
    Ok(state)
}
