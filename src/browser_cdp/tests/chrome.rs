use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::json;
use tungstenite::Message;

use super::super::chrome_process::ChromeProcess;
use super::super::client::CdpClient;
use super::super::discovery::connect_existing_profile_browser;
use super::super::page_scripts::{local_storage_set_expression, session_storage_set_expression};
use super::super::*;
use super::{remove_dir_all_with_retries, EnvVarGuard, ENV_LOCK};

#[test]
fn direct_page_cdp_connection_enables_domains_without_session_ids() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        for expected_method in [
            "Page.enable",
            "Runtime.enable",
            "Runtime.runIfWaitingForDebugger",
            "Network.enable",
        ] {
            let Message::Text(text) = websocket.read().unwrap() else {
                panic!("expected text CDP command");
            };
            let request: serde_json::Value = serde_json::from_str(text.as_ref()).unwrap();
            assert_eq!(request["method"], expected_method);
            assert!(request.get("sessionId").is_none());
            let id = request
                .get("id")
                .and_then(serde_json::Value::as_u64)
                .unwrap();
            websocket
                .send(Message::Text(
                    json!({ "id": id, "result": {} }).to_string().into(),
                ))
                .unwrap();
        }
        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/page/direct"),
        Duration::from_secs(2),
    )
    .unwrap();
    let page = client.attach_existing_page(Duration::from_secs(2)).unwrap();
    let page = page.expect("direct page connection should expose current page");
    assert!(page.target_id.is_empty());
    assert!(page.session_id.is_empty());
    client
        .enable_page_domains(&page.session_id, Duration::from_secs(2))
        .unwrap();
    handle.join().unwrap();
}

#[cfg(unix)]
#[test]
fn chrome_launch_retries_after_early_startup_exit() {
    use std::os::unix::fs::PermissionsExt;

    let _env_lock = ENV_LOCK.lock().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let profile = temp.path().join("profile");
    fs::create_dir_all(&profile).unwrap();
    let fake_chrome = temp.path().join("fake-chrome");
    let state_path = PathBuf::from(format!("{}.count", fake_chrome.display()));
    fs::write(
        &fake_chrome,
        "#!/bin/sh\n\
         state=\"$0.count\"\n\
         count=\"$(cat \"$state\" 2>/dev/null || echo 0)\"\n\
         count=$((count + 1))\n\
         printf '%s\\n' \"$count\" > \"$state\"\n\
         if [ \"$count\" -lt 2 ]; then\n\
           echo 'transient Chrome startup failure' >&2\n\
           exit 1\n\
         fi\n\
         user_data_dir=''\n\
         for arg in \"$@\"; do\n\
           case \"$arg\" in\n\
             --user-data-dir=*) user_data_dir=\"${arg#--user-data-dir=}\" ;;\n\
           esac\n\
         done\n\
         if [ -z \"$user_data_dir\" ]; then\n\
           echo 'missing user data dir' >&2\n\
           exit 2\n\
         fi\n\
         printf '49152\\n/devtools/browser/retry\\n' > \"$user_data_dir/DevToolsActivePort\"\n\
         sleep 60\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_chrome).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&fake_chrome, permissions).unwrap();

    let _env_guard = EnvVarGuard::set("AGET_CHROME_COMMAND", fake_chrome.as_os_str());
    let chrome = ChromeProcess::launch_profile(
        &profile,
        None,
        false,
        Duration::from_secs(2),
        "owned Chrome retry test",
    )
    .unwrap();

    assert_eq!(chrome.ws_url, "ws://127.0.0.1:49152/devtools/browser/retry");
    assert_eq!(fs::read_to_string(state_path).unwrap().trim(), "2");
}

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
