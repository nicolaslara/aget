use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;

use crate::browser_cdp::client::CdpClient;
use crate::browser_cdp::PageWaitUntil;

use super::super::super::{read_cdp_request, reply_ok};

#[test]
fn browser_cdp_navigation_error_text_is_reported() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Page.navigate");
        reply_ok(
            &mut websocket,
            &request,
            json!({
                "frameId": "frame-1",
                "loaderId": "loader-1",
                "errorText": "net::ERR_ABORTED"
            }),
        );
        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    let error = client
        .navigate_and_wait(
            "session-1",
            "https://example.com/fail",
            PageWaitUntil::Load,
            Duration::from_secs(2),
        )
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("Page.navigate failed: net::ERR_ABORTED"));
    handle.join().unwrap();
}

#[test]
fn browser_cdp_wait_for_selector_reports_runtime_evaluation_exception() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Runtime.evaluate");
        assert_eq!(request["sessionId"], "session-1");
        assert_eq!(request["params"]["returnByValue"], true);
        assert_eq!(request["params"]["awaitPromise"], false);
        assert!(request["params"]["expression"]
            .as_str()
            .unwrap()
            .contains("document.querySelector"));
        reply_ok(
            &mut websocket,
            &request,
            json!({
                "result": { "type": "object", "subtype": "error" },
                "exceptionDetails": {
                    "text": "Uncaught",
                    "exception": {
                        "description": "Error: selector script failed"
                    }
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
    let error = client
        .wait_for_selector("session-1", ".ready", Duration::from_secs(2))
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("Runtime.evaluate failed: Error: selector script failed"));
    handle.join().unwrap();
}

#[test]
fn browser_cdp_blank_response_navigation_reports_error_text() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Page.navigate");
        assert_eq!(request["sessionId"], "session-1");
        assert_eq!(request["params"]["url"], "https://blocked.example/");
        reply_ok(
            &mut websocket,
            &request,
            json!({
                "frameId": "frame-1",
                "loaderId": "loader-1",
                "errorText": "net::ERR_NAME_NOT_RESOLVED"
            }),
        );
        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    let error = client
        .navigate_with_blank_response(
            "session-1",
            "https://blocked.example/",
            Duration::from_secs(2),
        )
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("Page.navigate failed: net::ERR_NAME_NOT_RESOLVED"));
    handle.join().unwrap();
}
