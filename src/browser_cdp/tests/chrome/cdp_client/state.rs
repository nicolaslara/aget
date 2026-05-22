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
