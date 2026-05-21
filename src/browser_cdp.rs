mod chrome_process;
mod client;
mod discovery;
mod page_scripts;
mod process;
mod render;
mod state;

use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;
#[cfg(test)]
use std::{env, path::PathBuf, time::SystemTime, time::UNIX_EPOCH};

use self::chrome_process::ChromeProcess;
use self::client::CdpClient;
#[cfg(test)]
use self::client::{
    cdp_cookies, origin_storage_from_runtime_result, playwright_cookies_from_cdp,
    preferred_page_target_id, storage_candidate_origins,
};
use self::discovery::{connect_existing_profile_browser, wait_for_profile_browser_shutdown};
#[cfg(test)]
use self::discovery::{
    devtools_ws_url_from_stderr, discover_cdp_ws_url, read_devtools_active_port,
    relevant_chrome_stderr, rewrite_cdp_ws_host,
};
#[cfg(test)]
use self::page_scripts::{
    local_storage_set_expression, rendered_overlay_cleanup_expression, selector_exists_expression,
    session_storage_set_expression, shadow_dom_attach_override_expression,
    shadow_dom_flatten_expression,
};
use self::process::ensure_login_browser_exited;
pub(crate) use self::render::{render_page, BrowserRenderRequest, PageWaitUntil};
pub(crate) use self::state::{export_browser_state, BrowserStateExportRequest};
use crate::error::{AgetError, ErrorCode};
use serde_json::json;

use crate::session::PlaywrightState;

const CHROME_SHUTDOWN_WAIT: Duration = Duration::from_secs(1);
const CHROME_SANDBOX_STARTUP_HINT: &str = "Hint: Chrome sandbox/namespace startup failure; in containers or VMs, set AGET_CHROME_COMMAND to a Chrome/Chromium executable that can run with --no-sandbox";
const CHROME_SILENT_STARTUP_HINT: &str = "Hint: Chrome exited without startup diagnostics; in containers or VMs, set AGET_CHROME_COMMAND to a Chrome/Chromium wrapper that can run with --no-sandbox";

pub(crate) struct BrowserLoginStartRequest<'a> {
    pub(crate) profile_dir: &'a Path,
    pub(crate) url: &'a str,
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
    let chrome = ChromeProcess::launch_login(
        request.profile_dir,
        request.url,
        request.timeout,
        "owned login start",
    )?;
    let pid = chrome.id();
    chrome.detach();
    Ok(StartedLoginBrowser { pid })
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
            let _ = client.send(
                "Target.closeTarget",
                Some(json!({ "targetId": page.target_id })),
                None,
                Duration::from_secs(1),
            );
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

fn io_aget_error(error: impl ToString) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}

fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    set_private_dir_permissions(path)
}

#[cfg(unix)]
fn set_private_dir_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};
    use std::thread;

    use super::discovery::{classify_chrome_startup_error, wait_for_devtools_active_port};
    use super::process::terminate_child;
    use super::*;
    use crate::process::{create_private_file, TempOutputFile};
    use crate::session::PlaywrightCookie;
    use serde_json::Value;
    use tungstenite::Message;

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &std::ffi::OsStr) -> Self {
            let previous = env::var_os(key);
            unsafe {
                env::set_var(key, value);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(previous) = &self.previous {
                    env::set_var(self.key, previous);
                } else {
                    env::remove_var(self.key);
                }
            }
        }
    }

    #[test]
    fn cdp_cookie_payload_preserves_browser_cookie_fields() {
        let payload = cdp_cookies(&[PlaywrightCookie {
            name: "sid".to_string(),
            value: "secret".to_string(),
            domain: ".example.com".to_string(),
            path: "/account".to_string(),
            expires: Some(1_800_000_000),
            http_only: true,
            secure: true,
            same_site: Some("Lax".to_string()),
        }]);

        assert_eq!(
            payload,
            vec![json!({
                "name": "sid",
                "value": "secret",
                "domain": ".example.com",
                "path": "/account",
                "expires": 1_800_000_000,
                "httpOnly": true,
                "secure": true,
                "sameSite": "Lax",
            })]
        );
    }

    #[test]
    fn parses_cdp_cookies_into_playwright_state_shape() {
        let cookies = playwright_cookies_from_cdp(&json!({
            "cookies": [
                {
                    "name": "sid",
                    "value": "secret",
                    "domain": ".example.com",
                    "path": "/",
                    "expires": 1800000000.9,
                    "httpOnly": true,
                    "secure": true,
                    "sameSite": "Lax"
                },
                {
                    "name": "session",
                    "value": "secret",
                    "domain": "example.com",
                    "path": "/",
                    "expires": -1
                }
            ]
        }));

        assert_eq!(cookies.len(), 2);
        assert_eq!(cookies[0].expires, Some(1_800_000_000));
        assert_eq!(cookies[1].expires, None);
        assert!(cookies[0].http_only);
        assert!(cookies[0].secure);
        assert_eq!(cookies[0].same_site.as_deref(), Some("Lax"));
    }

    #[test]
    fn storage_candidate_origins_cover_http_and_https_domains() {
        let origins = storage_candidate_origins(&[
            "Example.COM".to_string(),
            ".docs.example.com/".to_string(),
            "https://app.example.com".to_string(),
        ]);

        assert_eq!(
            origins,
            vec![
                "http://docs.example.com".to_string(),
                "http://example.com".to_string(),
                "https://app.example.com".to_string(),
                "https://docs.example.com".to_string(),
                "https://example.com".to_string(),
            ]
        );
    }

    #[test]
    fn parses_origin_storage_runtime_value() {
        let origin = origin_storage_from_runtime_result(&json!({
            "result": {
                "value": {
                    "origin": "https://example.com",
                    "localStorage": [
                        {"name": "token", "value": "secret"}
                    ],
                    "sessionStorage": [
                        {"name": "ignored", "value": "session-only"}
                    ]
                }
            }
        }))
        .unwrap();

        assert_eq!(origin.origin, "https://example.com");
        assert_eq!(origin.local_storage.len(), 1);
        assert_eq!(origin.local_storage[0].name, "token");
        assert_eq!(origin.session_storage.len(), 1);
        assert_eq!(origin.session_storage[0].name, "ignored");
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

    #[test]
    fn local_storage_expression_json_quotes_key_and_value() {
        let expression =
            local_storage_set_expression("token\"name", "line\nvalue</script>").unwrap();

        assert_eq!(
            expression,
            "localStorage.setItem(\"token\\\"name\", \"line\\nvalue</script>\")"
        );
    }

    #[test]
    fn session_storage_expression_json_quotes_key_and_value() {
        let expression =
            session_storage_set_expression("token\"name", "line\nvalue</script>").unwrap();

        assert_eq!(
            expression,
            "sessionStorage.setItem(\"token\\\"name\", \"line\\nvalue</script>\")"
        );
    }

    #[test]
    fn prefers_existing_non_internal_page_target() {
        let target_id = preferred_page_target_id(&[
            json!({
                "targetId": "chrome",
                "type": "page",
                "url": "chrome://new-tab-page/"
            }),
            json!({
                "targetId": "blank",
                "type": "page",
                "url": "about:blank"
            }),
            json!({
                "targetId": "login",
                "type": "page",
                "url": "https://example.com/login"
            }),
        ]);

        assert_eq!(target_id.as_deref(), Some("login"));
    }

    #[test]
    fn selector_wait_expression_json_quotes_css_selector() {
        let expression = selector_exists_expression(r#"main[data-name="a b"]"#).unwrap();

        assert_eq!(
            expression,
            r#"document.querySelector("main[data-name=\"a b\"]") !== null"#
        );
    }

    #[test]
    fn rendered_overlay_cleanup_expression_uses_generic_crawl4ai_rules() {
        let expression = rendered_overlay_cleanup_expression();

        assert!(expression.contains("cookie-banner"));
        assert!(expression.contains("cookie-consent"));
        assert!(expression.contains("newsletter"));
        assert!(expression.contains("role=\"dialog\""));
        assert!(expression.contains("style.position === \"fixed\""));
        assert!(expression.contains("style.position === \"absolute\""));
        assert!(expression.contains("zIndex > 999"));
        assert!(!expression.contains("hellointerview"));
    }

    #[test]
    fn shadow_dom_flatten_expression_resolves_slots_and_skips_styles() {
        let attach_override = shadow_dom_attach_override_expression();
        let flatten = shadow_dom_flatten_expression();

        assert!(attach_override.contains("attachShadow"));
        assert!(attach_override.contains("mode: \"open\""));
        assert!(flatten.contains("shadowRoot"));
        assert!(flatten.contains("assignedNodes({ flatten: true })"));
        assert!(flatten.contains("tag === \"style\""));
        assert!(flatten.contains("serialize(document.documentElement)"));
    }

    #[test]
    fn parses_devtools_ws_url_from_chrome_stderr() {
        let stderr = "\
noise
DevTools listening on ws://127.0.0.1:9222/devtools/browser/fallback
more noise";

        assert_eq!(
            devtools_ws_url_from_stderr(stderr).as_deref(),
            Some("ws://127.0.0.1:9222/devtools/browser/fallback")
        );
    }

    #[test]
    fn chrome_stderr_detail_includes_sandbox_hint() {
        let detail = relevant_chrome_stderr(
            "Failed to move to new namespace: PID namespaces supported, Network namespace supported, but failed: errno = Operation not permitted",
        );

        assert!(detail.contains("Failed to move to new namespace"));
        assert!(detail.contains("Chrome sandbox/namespace startup failure"));
        assert!(detail.contains("AGET_CHROME_COMMAND"));
    }

    #[test]
    fn chrome_startup_error_adds_silent_exit_hint_without_stderr() {
        let temp = tempfile::tempdir().unwrap();
        let stderr_capture = TempOutputFile::new(temp.path(), "chrome-stderr").unwrap();
        let error = AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: "owned browser fallback Chrome exited before CDP startup".to_string(),
        };

        let classified = classify_chrome_startup_error("owned Chrome test", error, &stderr_capture);

        assert_eq!(classified.code(), ErrorCode::BackendUnavailable);
        let message = classified.to_string();
        assert!(message.contains("Chrome exited without startup diagnostics"));
        assert!(message.contains("AGET_CHROME_COMMAND"));
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_devtools_active_port_uses_stderr_fallback() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let stderr_capture = TempOutputFile::new(temp.path(), "chrome-stderr").unwrap();
        let stderr = create_private_file(stderr_capture.path()).unwrap();
        let fake_chrome = temp.path().join("fake-chrome");
        fs::write(
            &fake_chrome,
            "#!/bin/sh\n\
             echo 'DevTools listening on ws://127.0.0.1:9222/devtools/browser/fallback' >&2\n\
             sleep 5\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fake_chrome).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&fake_chrome, permissions).unwrap();
        let mut child = Command::new(&fake_chrome)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(stderr))
            .spawn()
            .unwrap();

        let ws_url = wait_for_devtools_active_port(
            &mut child,
            temp.path(),
            &stderr_capture,
            Duration::from_secs(2),
        )
        .unwrap();
        terminate_child(&mut child);

        assert_eq!(ws_url, "ws://127.0.0.1:9222/devtools/browser/fallback");
    }

    #[test]
    fn parses_devtools_active_port_file() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(
            temp.path().join("DevToolsActivePort"),
            "49152\n/devtools/browser/abc\n",
        )
        .unwrap();

        assert_eq!(
            read_devtools_active_port(temp.path()),
            Some((49152, "/devtools/browser/abc".to_string()))
        );
    }

    #[test]
    fn existing_profile_attach_removes_stale_devtools_active_port() {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let temp = tempfile::tempdir().unwrap();
        let active_port = temp.path().join("DevToolsActivePort");
        fs::write(&active_port, format!("{port}\n/devtools/browser/stale\n")).unwrap();

        let client = connect_existing_profile_browser(temp.path(), Duration::from_millis(100))
            .expect("stale CDP attach should not be fatal");

        assert!(client.is_none());
        assert!(!active_port.exists());
    }

    #[test]
    fn rewrites_discovered_cdp_websocket_host_and_port() {
        assert_eq!(
            rewrite_cdp_ws_host("ws://localhost:9222/devtools/browser/abc", 49152).as_deref(),
            Some("ws://127.0.0.1:49152/devtools/browser/abc")
        );
        assert!(rewrite_cdp_ws_host("http://localhost:9222/json/version", 49152).is_none());
    }

    #[test]
    fn discovers_cdp_websocket_url_from_json_version() {
        let body = r#"{"webSocketDebuggerUrl":"ws://localhost:9999/devtools/browser/discovered"}"#;
        let (port, handle) = serve_cdp_discovery_responses(vec![http_json_response(200, body)]);

        let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
        handle.join().unwrap();

        assert_eq!(
            ws_url,
            format!("ws://127.0.0.1:{port}/devtools/browser/discovered")
        );
    }

    #[test]
    fn discovers_cdp_websocket_url_from_json_list_fallback() {
        let list_body = r#"[
            {
                "type": "page",
                "webSocketDebuggerUrl": "ws://localhost:9999/devtools/page/ignored"
            },
            {
                "type": "browser",
                "webSocketDebuggerUrl": "ws://localhost:9999/devtools/browser/list"
            }
        ]"#;
        let (port, handle) = serve_cdp_discovery_responses(vec![
            http_json_response(200, r#"{}"#),
            http_json_response(200, list_body),
        ]);

        let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
        handle.join().unwrap();

        assert_eq!(
            ws_url,
            format!("ws://127.0.0.1:{port}/devtools/browser/list")
        );
    }

    #[test]
    fn discovers_cdp_websocket_url_from_first_list_target_with_ws() {
        let list_body = r#"[
            {
                "type": "page"
            },
            {
                "type": "page",
                "webSocketDebuggerUrl": "ws://localhost:9999/devtools/page/fallback"
            }
        ]"#;
        let (port, handle) = serve_cdp_discovery_responses(vec![
            http_json_response(404, r#"{"error":"missing"}"#),
            http_json_response(200, list_body),
        ]);

        let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
        handle.join().unwrap();

        assert_eq!(
            ws_url,
            format!("ws://127.0.0.1:{port}/devtools/page/fallback")
        );
    }

    #[test]
    fn discovers_cdp_websocket_url_from_direct_websocket_fallback() {
        use std::io::{Read as _, Write as _};
        use std::net::TcpListener;

        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0u8; 512];
                let _ = stream.read(&mut request);
                let response = http_json_response(404, r#"{"error":"missing"}"#);
                stream.write_all(response.as_bytes()).unwrap();
            }

            let (stream, _) = listener.accept().unwrap();
            let mut websocket = tungstenite::accept(stream).unwrap();
            let Message::Text(text) = websocket.read().unwrap() else {
                panic!("expected text CDP command");
            };
            let request: Value = serde_json::from_str(text.as_ref()).unwrap();
            let id = request.get("id").and_then(Value::as_u64).unwrap();
            let reply = json!({
                "id": id,
                "result": {
                    "protocolVersion": "1.3",
                    "product": "Chrome/136"
                }
            })
            .to_string();
            websocket.send(Message::Text(reply.into())).unwrap();
            let _ = websocket.close(None);
        });

        let ws_url = discover_cdp_ws_url(port, Duration::from_secs(2)).unwrap();
        handle.join().unwrap();

        assert_eq!(ws_url, format!("ws://127.0.0.1:{port}/devtools/browser"));
    }

    fn serve_cdp_discovery_responses(responses: Vec<String>) -> (u16, thread::JoinHandle<()>) {
        use std::io::{Read as _, Write as _};
        use std::net::TcpListener;

        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0u8; 512];
                let _ = stream.read(&mut request);
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        (port, handle)
    }

    fn http_json_response(status: u16, body: &str) -> String {
        let reason = match status {
            200 => "OK",
            404 => "Not Found",
            _ => "Status",
        };
        format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
    }

    fn remove_dir_all_with_retries(path: &Path) -> io::Result<()> {
        let mut last_error = None;
        for _ in 0..5 {
            match fs::remove_dir_all(path) {
                Ok(()) => return Ok(()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
                Err(error) => {
                    last_error = Some(error);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
        Err(last_error.unwrap_or_else(|| io::Error::other("remove_dir_all failed")))
    }
}
