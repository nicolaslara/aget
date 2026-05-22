use std::path::Path;
use std::time::Duration;

use crate::error::AgetError;
use crate::session::PlaywrightState;

use super::chrome_process::ChromeProcess;
use super::client::CdpClient;
use super::discovery::{connect_existing_profile_browser, wait_for_profile_browser_shutdown};
use super::process::ensure_login_browser_exited;
use super::render::PageWaitUntil;
use super::state::{export_browser_state, BrowserStateExportRequest};
use super::{create_private_dir, io_aget_error};

pub(crate) struct BrowserLoginStartRequest<'a> {
    pub(crate) profile_dir: &'a Path,
    pub(crate) url: &'a str,
    pub(crate) state: &'a PlaywrightState,
    pub(crate) timeout: Duration,
}

pub(crate) struct StartedLoginBrowser {
    pub(crate) pid: u32,
}

pub(crate) struct BrowserLoginStateExportRequest<'a> {
    pub(crate) profile_dir: &'a Path,
    pub(crate) allowed_domains: &'a [String],
    pub(crate) pid: Option<u32>,
    pub(crate) timeout: Duration,
}

pub(crate) struct BrowserLoginCloseRequest<'a> {
    pub(crate) profile_dir: &'a Path,
    pub(crate) pid: Option<u32>,
    pub(crate) timeout: Duration,
}

pub(crate) fn start_login_browser(
    request: BrowserLoginStartRequest<'_>,
) -> Result<StartedLoginBrowser, AgetError> {
    create_private_dir(request.profile_dir).map_err(io_aget_error)?;
    let startup_url = if request.state.cookies.is_empty() && request.state.origins.is_empty() {
        request.url
    } else {
        "about:blank"
    };
    let chrome = ChromeProcess::launch_login(
        request.profile_dir,
        startup_url,
        request.timeout,
        "owned login start",
    )?;
    if !request.state.cookies.is_empty() || !request.state.origins.is_empty() {
        seed_login_browser_state(&chrome, request)?;
    }
    let pid = chrome.id();
    chrome.detach();
    Ok(StartedLoginBrowser { pid })
}

fn seed_login_browser_state(
    chrome: &ChromeProcess,
    request: BrowserLoginStartRequest<'_>,
) -> Result<(), AgetError> {
    let mut client = CdpClient::connect(&chrome.ws_url, request.timeout)?;
    let page = match client.attach_existing_page(request.timeout)? {
        Some(page) => page,
        None => client.create_page(request.timeout)?,
    };
    client.enable_page_domains(&page.session_id, request.timeout)?;
    client.load_state(&page.session_id, request.state, request.timeout)?;
    client.navigate_and_wait(
        &page.session_id,
        request.url,
        PageWaitUntil::Load,
        request.timeout,
    )
}

pub(crate) fn export_login_browser_state(
    request: BrowserLoginStateExportRequest<'_>,
) -> Result<PlaywrightState, AgetError> {
    if let Some(mut client) =
        connect_existing_profile_browser(request.profile_dir, request.timeout)?
    {
        let existing_page = client.attach_existing_page(request.timeout)?;
        let created_page = existing_page.is_none();
        let page = match existing_page {
            Some(page) => page,
            None => client.create_page(request.timeout)?,
        };
        client.enable_page_domains(&page.session_id, request.timeout)?;
        let state =
            client.export_state(&page.session_id, request.allowed_domains, request.timeout)?;
        if created_page {
            let _ = client.close_page(&page, Duration::from_secs(1));
        }
        client.close_browser(request.timeout)?;
        wait_for_profile_browser_shutdown(request.profile_dir, Duration::from_secs(5));
        ensure_login_browser_exited(request.profile_dir, request.pid, Duration::from_secs(5))?;
        return Ok(state);
    }

    ensure_login_browser_exited(request.profile_dir, request.pid, Duration::from_secs(5))?;

    export_browser_state(BrowserStateExportRequest {
        profile_dir: request.profile_dir,
        profile_directory: None,
        use_real_keychain: false,
        allowed_domains: request.allowed_domains,
        timeout: request.timeout,
    })
}

pub(crate) fn close_login_browser(request: BrowserLoginCloseRequest<'_>) -> Result<(), AgetError> {
    if let Some(mut client) =
        connect_existing_profile_browser(request.profile_dir, request.timeout)?
    {
        client.close_browser(request.timeout)?;
        wait_for_profile_browser_shutdown(request.profile_dir, Duration::from_secs(5));
    }
    ensure_login_browser_exited(request.profile_dir, request.pid, Duration::from_secs(5))?;
    Ok(())
}
