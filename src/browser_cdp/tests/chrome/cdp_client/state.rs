use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;

use crate::browser_cdp::client::CdpClient;
use crate::session::{PlaywrightOrigin, PlaywrightState, StorageEntry};

use super::super::{read_cdp_request, reply_ok};

#[test]
fn browser_cdp_load_state_reports_storage_runtime_evaluation_exception() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Page.navigate");
        assert_eq!(request["sessionId"], "session-1");
        assert_eq!(request["params"]["url"], "https://example.com/");
        reply_ok(&mut websocket, &request, json!({ "frameId": "frame-1" }));

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Runtime.evaluate");
        assert_eq!(request["sessionId"], "session-1");
        assert_eq!(request["params"]["returnByValue"], true);
        assert_eq!(request["params"]["awaitPromise"], false);
        let expression = request["params"]["expression"].as_str().unwrap();
        assert!(expression.contains("localStorage.setItem"));
        assert!(expression.contains("\"token\""));
        reply_ok(
            &mut websocket,
            &request,
            json!({
                "result": { "type": "object", "subtype": "error" },
                "exceptionDetails": {
                    "text": "SecurityError: localStorage is disabled"
                }
            }),
        );

        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    let state = PlaywrightState {
        cookies: Vec::new(),
        origins: vec![PlaywrightOrigin {
            origin: "https://example.com".to_string(),
            local_storage: vec![StorageEntry {
                name: "token".to_string(),
                value: "secret".to_string(),
            }],
            session_storage: Vec::new(),
        }],
    };
    let error = client
        .load_state("session-1", &state, Duration::from_secs(2))
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("Runtime.evaluate failed: SecurityError: localStorage is disabled"));
    handle.join().unwrap();
}

#[test]
fn browser_cdp_export_state_collects_allowed_frame_storage_origins() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Network.getAllCookies");
        assert_eq!(request["sessionId"], "session-1");
        reply_ok(&mut websocket, &request, json!({ "cookies": [] }));

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Network.getCookies");
        assert_eq!(request["sessionId"], "session-1");
        assert_eq!(
            request["params"]["urls"],
            json!(["http://app.example.com/", "https://app.example.com/"])
        );
        reply_ok(&mut websocket, &request, json!({ "cookies": [] }));

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Storage.getCookies");
        assert_eq!(request["sessionId"], "session-1");
        reply_ok(&mut websocket, &request, json!({ "cookies": [] }));

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Page.getFrameTree");
        assert_eq!(request["sessionId"], "session-1");
        reply_ok(
            &mut websocket,
            &request,
            json!({
                "frameTree": {
                    "frame": { "url": "https://app.example.com/account" },
                    "childFrames": [
                        { "frame": { "url": "https://auth.app.example.com/oauth" } },
                        { "frame": { "url": "https://evil.example.test/frame" } }
                    ]
                }
            }),
        );

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Fetch.enable");
        assert_eq!(request["sessionId"], "session-1");
        reply_ok(&mut websocket, &request, json!({}));

        for expected_url in [
            "http://app.example.com/",
            "https://app.example.com/",
            "https://auth.app.example.com/",
        ] {
            let request = read_cdp_request(&mut websocket);
            assert_eq!(request["method"], "Page.navigate");
            assert_eq!(request["sessionId"], "session-1");
            assert_eq!(request["params"]["url"], expected_url);
            reply_ok(&mut websocket, &request, json!({ "frameId": "frame-1" }));

            let request = read_cdp_request(&mut websocket);
            assert_eq!(request["method"], "Runtime.evaluate");
            assert_eq!(request["sessionId"], "session-1");
            assert!(request["params"]["expression"]
                .as_str()
                .unwrap()
                .contains("localStorage"));
            let storage = if expected_url == "https://auth.app.example.com/" {
                json!({
                    "origin": "https://auth.app.example.com",
                    "localStorage": [{ "name": "oauth", "value": "provider-token" }],
                    "sessionStorage": [{ "name": "nonce", "value": "session-token" }]
                })
            } else {
                json!({
                    "origin": expected_url.trim_end_matches('/'),
                    "localStorage": [],
                    "sessionStorage": []
                })
            };
            reply_ok(
                &mut websocket,
                &request,
                json!({ "result": { "type": "object", "value": storage } }),
            );
        }

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Fetch.disable");
        assert_eq!(request["sessionId"], "session-1");
        reply_ok(&mut websocket, &request, json!({}));

        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    let state = client
        .export_state(
            "session-1",
            &["app.example.com".to_string()],
            Duration::from_secs(2),
        )
        .unwrap();

    assert!(state.cookies.is_empty());
    assert_eq!(state.origins.len(), 1);
    assert_eq!(state.origins[0].origin, "https://auth.app.example.com");
    assert_eq!(state.origins[0].local_storage[0].name, "oauth");
    assert_eq!(state.origins[0].local_storage[0].value, "provider-token");
    assert_eq!(state.origins[0].session_storage[0].name, "nonce");
    assert_eq!(state.origins[0].session_storage[0].value, "session-token");
    handle.join().unwrap();
}
