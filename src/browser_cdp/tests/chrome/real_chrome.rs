use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::json;

use super::super::super::chrome_process::ChromeProcess;
use super::super::super::client::CdpClient;
use super::super::super::discovery::connect_existing_profile_browser;
use super::super::super::page_scripts::{
    local_storage_set_expression, session_storage_set_expression,
};
use super::super::super::*;
use super::super::remove_dir_all_with_retries;

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory() {
    let temp = tempfile::tempdir().unwrap();
    let profile = temp.path().join("profile");
    fs::create_dir_all(&profile).unwrap();
    let profile_directory = "Profile 1";
    let timeout = Duration::from_secs(20);
    let expires = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;

    {
        let mut chrome = ChromeProcess::launch_profile(
            &profile,
            Some(profile_directory),
            false,
            timeout,
            "owned Chrome test",
        )
        .expect("Chrome should launch for test profile");
        let mut client = CdpClient::connect(&chrome.ws_url, timeout).unwrap();
        let page = client.create_page(timeout).unwrap();
        client
            .enable_page_domains(&page.session_id, timeout)
            .unwrap();
        client
            .send(
                "Fetch.enable",
                Some(json!({ "patterns": [{ "urlPattern": "*" }] })),
                Some(&page.session_id),
                timeout,
            )
            .unwrap();
        client
            .navigate_with_blank_response(&page.session_id, "https://example.com/", timeout)
            .unwrap();
        client
            .send(
                "Network.setCookie",
                Some(json!({
                    "name": "sid",
                    "value": "secret",
                    "url": "https://example.com/",
                    "path": "/",
                    "expires": expires,
                    "secure": true,
                    "sameSite": "Lax",
                })),
                Some(&page.session_id),
                timeout,
            )
            .unwrap();
        client
            .send(
                "Runtime.evaluate",
                Some(json!({
                    "expression": local_storage_set_expression("token", "storage-secret").unwrap(),
                    "returnByValue": true,
                    "awaitPromise": false,
                })),
                Some(&page.session_id),
                timeout,
            )
            .unwrap();
        client
            .send(
                "Runtime.evaluate",
                Some(json!({
                    "expression": session_storage_set_expression("session-token", "session-secret").unwrap(),
                    "returnByValue": true,
                    "awaitPromise": false,
                })),
                Some(&page.session_id),
                timeout,
            )
            .unwrap();
        let _ = client.send("Browser.close", None, None, Duration::from_secs(1));
        chrome.wait_or_kill(Duration::from_secs(5));
    }

    let exported = export_browser_state(BrowserStateExportRequest {
        profile_dir: &profile,
        profile_directory: Some(profile_directory),
        use_real_keychain: false,
        allowed_domains: &["example.com".to_string()],
        timeout,
    })
    .unwrap();

    assert!(exported
        .cookies
        .iter()
        .any(|cookie| cookie.name == "sid" && cookie.value == "secret"));
    assert!(exported.origins.iter().any(|origin| {
        origin.origin == "https://example.com"
            && origin
                .local_storage
                .iter()
                .any(|entry| entry.name == "token" && entry.value == "storage-secret")
    }));
}

#[test]
#[ignore = "requires local Chrome/Chromium and opens a visible browser window"]
fn owned_login_browser_exports_state_from_headed_profile_and_closes() {
    let temp = tempfile::tempdir().unwrap();
    let profile = temp.path().join("login-profile");
    let timeout = Duration::from_secs(20);
    let expires = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;

    let started = start_login_browser(BrowserLoginStartRequest {
        profile_dir: &profile,
        url: "https://example.com/",
        timeout,
    })
    .expect("Chrome should launch for headed login test");

    {
        let mut client = connect_existing_profile_browser(&profile, timeout)
            .unwrap()
            .expect("headed login browser should expose CDP");
        let page = client
            .attach_existing_page(timeout)
            .unwrap()
            .unwrap_or_else(|| client.create_page(timeout).unwrap());
        client
            .enable_page_domains(&page.session_id, timeout)
            .unwrap();
        client
            .send(
                "Fetch.enable",
                Some(json!({ "patterns": [{ "urlPattern": "*" }] })),
                Some(&page.session_id),
                timeout,
            )
            .unwrap();
        client
            .navigate_with_blank_response(&page.session_id, "https://example.com/", timeout)
            .unwrap();
        client
            .send(
                "Network.setCookie",
                Some(json!({
                    "name": "sid",
                    "value": "login-secret",
                    "url": "https://example.com/",
                    "path": "/",
                    "expires": expires,
                    "secure": true,
                    "sameSite": "Lax",
                })),
                Some(&page.session_id),
                timeout,
            )
            .unwrap();
        client
            .send(
                "Runtime.evaluate",
                Some(json!({
                    "expression": local_storage_set_expression("token", "login-storage").unwrap(),
                    "returnByValue": true,
                    "awaitPromise": false,
                })),
                Some(&page.session_id),
                timeout,
            )
            .unwrap();
        client
            .send(
                "Runtime.evaluate",
                Some(json!({
                    "expression": session_storage_set_expression("session-token", "login-session").unwrap(),
                    "returnByValue": true,
                    "awaitPromise": false,
                })),
                Some(&page.session_id),
                timeout,
            )
            .unwrap();
    }

    let exported = export_login_browser_state(BrowserLoginStateExportRequest {
        profile_dir: &profile,
        allowed_domains: &["example.com".to_string()],
        pid: Some(started.pid),
        timeout,
    })
    .unwrap();

    assert!(exported
        .cookies
        .iter()
        .any(|cookie| cookie.name == "sid" && cookie.value == "login-secret"));
    assert!(exported.origins.iter().any(|origin| {
        origin.origin == "https://example.com"
            && origin
                .local_storage
                .iter()
                .any(|entry| entry.name == "token" && entry.value == "login-storage")
            && origin
                .session_storage
                .iter()
                .any(|entry| entry.name == "session-token" && entry.value == "login-session")
    }));
    remove_dir_all_with_retries(&profile).unwrap();
}
