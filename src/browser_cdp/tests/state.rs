use serde_json::json;

use super::super::client::{
    cdp_cookies, frame_storage_candidate_origins, origin_storage_from_runtime_result,
    playwright_cookies_from_cdp, storage_candidate_origins,
};
use crate::session::PlaywrightCookie;

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
fn frame_storage_candidate_origins_keep_explicit_allow_domain_scope() {
    let origins = frame_storage_candidate_origins(
        &json!({
            "frameTree": {
                "frame": { "url": "https://app.example.com/account" },
                "childFrames": [
                    { "frame": { "url": "https://auth.app.example.com/oauth" } },
                    { "frame": { "url": "https://evil.example.test/frame" } },
                    { "frame": { "url": "about:blank" } }
                ]
            }
        }),
        &["app.example.com".to_string()],
    );

    assert_eq!(
        origins,
        vec![
            "https://app.example.com".to_string(),
            "https://auth.app.example.com".to_string(),
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
