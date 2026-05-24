use super::*;
use crate::error::ErrorCode;
use crate::session::{PlaywrightCookie, PlaywrightOrigin, SessionSource, StorageEntry};

#[test]
fn filters_cookies_and_origins_by_allowed_domains() {
    let state = BrowserState {
        cookies: vec![
            cookie("sid", "example.com"),
            cookie("sub", "docs.example.com"),
            cookie("evil", "example.com.evil"),
        ],
        origins: vec![
            origin("https://example.com"),
            origin("https://docs.example.com:443"),
            origin("https://example.com.evil"),
        ],
    };

    let session = filter_browser_state(state, filter()).unwrap();

    assert_eq!(session.cookies.len(), 2);
    assert!(session.cookies.iter().any(|cookie| cookie.name == "sid"));
    assert!(session.cookies.iter().any(|cookie| cookie.name == "sub"));
    assert!(!session.cookies.iter().any(|cookie| cookie.name == "evil"));
    assert_eq!(session.origins.len(), 2);
    assert!(session
        .origins
        .iter()
        .any(|origin| origin.origin == "https://example.com"));
    assert!(session
        .origins
        .iter()
        .any(|origin| origin.origin == "https://docs.example.com:443"));
    assert!(session.origins.iter().any(|origin| {
        origin.origin == "https://example.com"
            && origin
                .session_storage
                .iter()
                .any(|entry| entry.name == "session-token")
    }));
    assert_eq!(
        session.source,
        SessionSource::ChromeProfile {
            profile: "Default".to_string()
        }
    );
    assert!(session.sensitive);
}

#[test]
fn rejects_conflicting_duplicate_cookies() {
    let mut duplicate = cookie("sid", "example.com");
    duplicate.value = "different-secret".to_string();
    let state = BrowserState {
        cookies: vec![cookie("sid", "example.com"), duplicate],
        origins: Vec::new(),
    };

    let error = filter_browser_state(state, filter()).unwrap_err();

    assert_eq!(error.code(), ErrorCode::SessionConflict);
}

#[test]
fn rejects_conflicting_duplicate_origins() {
    let mut duplicate = origin("https://example.com");
    duplicate.local_storage = vec![StorageEntry {
        name: "token".to_string(),
        value: "different-secret".to_string(),
    }];
    let state = BrowserState {
        cookies: Vec::new(),
        origins: vec![origin("https://example.com"), duplicate],
    };

    let error = filter_browser_state(state, filter()).unwrap_err();

    assert_eq!(error.code(), ErrorCode::SessionConflict);
}

#[test]
fn filters_playwright_state_with_same_import_rules() {
    let state = crate::session::PlaywrightState {
        cookies: vec![
            PlaywrightCookie {
                name: "sid".to_string(),
                value: "secret".to_string(),
                domain: ".example.com".to_string(),
                path: "/".to_string(),
                expires: Some(1_800_000_000),
                http_only: true,
                secure: true,
                same_site: Some("Lax".to_string()),
            },
            PlaywrightCookie {
                name: "evil".to_string(),
                value: "secret".to_string(),
                domain: "example.com.evil".to_string(),
                path: "/".to_string(),
                expires: None,
                http_only: false,
                secure: false,
                same_site: None,
            },
        ],
        origins: vec![
            PlaywrightOrigin {
                origin: "https://example.com".to_string(),
                local_storage: vec![StorageEntry {
                    name: "token".to_string(),
                    value: "secret".to_string(),
                }],
                session_storage: vec![StorageEntry {
                    name: "session-token".to_string(),
                    value: "session-secret".to_string(),
                }],
            },
            PlaywrightOrigin {
                origin: "https://example.com.evil".to_string(),
                local_storage: vec![StorageEntry {
                    name: "token".to_string(),
                    value: "evil".to_string(),
                }],
                session_storage: vec![StorageEntry {
                    name: "session-token".to_string(),
                    value: "evil-session".to_string(),
                }],
            },
        ],
    };

    let session = filter_playwright_state(state, filter()).unwrap();

    assert_eq!(session.cookies.len(), 1);
    assert_eq!(session.cookies[0].name, "sid");
    assert_eq!(session.origins.len(), 1);
    assert_eq!(session.origins[0].origin, "https://example.com");
    assert_eq!(session.origins[0].session_storage.len(), 1);
    assert_eq!(session.origins[0].session_storage[0].name, "session-token");
}

#[test]
fn origin_host_parses_http_hosts() {
    assert_eq!(
        origin_host("https://example.com:443"),
        Some("example.com".to_string())
    );
    assert_eq!(
        origin_host("https://user@example.com"),
        Some("example.com".to_string())
    );
    assert_eq!(origin_host("file:///tmp/page.html"), None);
}

fn filter() -> BrowserSessionFilter {
    BrowserSessionFilter {
        name: "demo".to_string(),
        source: SessionSource::ChromeProfile {
            profile: "Default".to_string(),
        },
        allowed_domains: vec!["example.com".to_string()],
        source_session: "aget-import-test".to_string(),
    }
}

fn cookie(name: &str, domain: &str) -> BrowserCookie {
    BrowserCookie {
        name: name.to_string(),
        value: format!("{name}-secret"),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some("Lax".to_string()),
    }
}

fn origin(origin: &str) -> BrowserOrigin {
    BrowserOrigin {
        origin: origin.to_string(),
        local_storage: vec![StorageEntry {
            name: "token".to_string(),
            value: "secret".to_string(),
        }],
        session_storage: vec![StorageEntry {
            name: "session-token".to_string(),
            value: "session-secret".to_string(),
        }],
    }
}
