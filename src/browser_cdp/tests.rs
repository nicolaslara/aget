use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, ffi};

use super::chrome_process::ChromeProcess;
use super::client::{
    cdp_cookies, origin_storage_from_runtime_result, playwright_cookies_from_cdp,
    preferred_page_target_id, storage_candidate_origins, CdpClient,
};
use super::discovery::connect_existing_profile_browser;
use super::page_scripts::{
    local_storage_set_expression, rendered_overlay_cleanup_expression, selector_exists_expression,
    session_storage_set_expression, shadow_dom_attach_override_expression,
    shadow_dom_flatten_expression,
};
use super::*;
use crate::session::PlaywrightCookie;
use serde_json::json;

mod discovery;

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct EnvVarGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &ffi::OsStr) -> Self {
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
    let expression = local_storage_set_expression("token\"name", "line\nvalue</script>").unwrap();

    assert_eq!(
        expression,
        "localStorage.setItem(\"token\\\"name\", \"line\\nvalue</script>\")"
    );
}

#[test]
fn session_storage_expression_json_quotes_key_and_value() {
    let expression = session_storage_set_expression("token\"name", "line\nvalue</script>").unwrap();

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
