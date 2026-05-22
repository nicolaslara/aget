use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;
use tungstenite::Message;

use crate::browser_cdp::client::CdpClient;
use crate::browser_cdp::PageWaitUntil;

use super::super::{read_cdp_request, reply_ok};

#[test]
fn browser_cdp_same_document_navigation_does_not_wait_for_lifecycle_events() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Page.navigate");
        assert_eq!(request["sessionId"], "session-1");
        assert_eq!(request["params"]["url"], "https://example.com/page#section");
        reply_ok(&mut websocket, &request, json!({ "frameId": "frame-1" }));
        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    client
        .navigate_and_wait(
            "session-1",
            "https://example.com/page#section",
            PageWaitUntil::NetworkIdle,
            Duration::from_secs(2),
        )
        .unwrap();
    handle.join().unwrap();
}

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
fn browser_cdp_lifecycle_wait_timeout_reports_wait_condition() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Page.navigate");
        assert_eq!(request["sessionId"], "session-1");
        reply_ok(
            &mut websocket,
            &request,
            json!({ "frameId": "frame-1", "loaderId": "loader-1" }),
        );
        thread::sleep(Duration::from_millis(450));
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
            "https://example.com/slow-load",
            PageWaitUntil::Load,
            Duration::from_millis(200),
        )
        .unwrap_err();
    assert_eq!(error.code(), crate::error::ErrorCode::Timeout);
    assert!(error.to_string().contains("timed out waiting for load"));
    handle.join().unwrap();
}

#[test]
fn browser_cdp_networkidle_can_start_from_load_event() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Page.navigate");
        assert_eq!(request["sessionId"], "session-1");
        reply_ok(
            &mut websocket,
            &request,
            json!({ "frameId": "frame-1", "loaderId": "loader-1" }),
        );
        websocket
            .send(Message::Text(
                json!({
                    "method": "Page.loadEventFired",
                    "sessionId": "session-1",
                    "params": {}
                })
                .to_string()
                .into(),
            ))
            .unwrap();
        thread::sleep(Duration::from_millis(700));
        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    client
        .navigate_and_wait(
            "session-1",
            "https://example.com/load-only",
            PageWaitUntil::NetworkIdle,
            Duration::from_secs(2),
        )
        .unwrap();
    handle.join().unwrap();
}

#[test]
fn browser_cdp_networkidle_resets_when_request_arrives_during_idle_window() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Page.navigate");
        assert_eq!(request["sessionId"], "session-1");
        reply_ok(
            &mut websocket,
            &request,
            json!({ "frameId": "frame-1", "loaderId": "loader-1" }),
        );
        websocket
            .send(Message::Text(
                json!({
                    "method": "Page.loadEventFired",
                    "sessionId": "session-1",
                    "params": {}
                })
                .to_string()
                .into(),
            ))
            .unwrap();
        thread::sleep(Duration::from_millis(200));
        websocket
            .send(Message::Text(
                json!({
                    "method": "Network.requestWillBeSent",
                    "sessionId": "session-1",
                    "params": { "requestId": "r1" }
                })
                .to_string()
                .into(),
            ))
            .unwrap();
        thread::sleep(Duration::from_millis(100));
        websocket
            .send(Message::Text(
                json!({
                    "method": "Network.loadingFinished",
                    "sessionId": "session-1",
                    "params": { "requestId": "r1" }
                })
                .to_string()
                .into(),
            ))
            .unwrap();
        thread::sleep(Duration::from_millis(700));
        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    let start = Instant::now();
    client
        .navigate_and_wait(
            "session-1",
            "https://example.com/delayed-fetch",
            PageWaitUntil::NetworkIdle,
            Duration::from_secs(2),
        )
        .unwrap();
    assert!(
        start.elapsed() >= Duration::from_millis(750),
        "networkidle resolved before the reset idle window elapsed"
    );
    handle.join().unwrap();
}

#[test]
fn browser_cdp_networkidle_times_out_when_requests_never_settle() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Page.navigate");
        assert_eq!(request["sessionId"], "session-1");
        reply_ok(
            &mut websocket,
            &request,
            json!({ "frameId": "frame-1", "loaderId": "loader-1" }),
        );
        websocket
            .send(Message::Text(
                json!({
                    "method": "Page.loadEventFired",
                    "sessionId": "session-1",
                    "params": {}
                })
                .to_string()
                .into(),
            ))
            .unwrap();
        websocket
            .send(Message::Text(
                json!({
                    "method": "Network.requestWillBeSent",
                    "sessionId": "session-1",
                    "params": { "requestId": "r1" }
                })
                .to_string()
                .into(),
            ))
            .unwrap();
        thread::sleep(Duration::from_millis(600));
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
            "https://example.com/long-poll",
            PageWaitUntil::NetworkIdle,
            Duration::from_millis(350),
        )
        .unwrap_err();
    assert_eq!(error.code(), crate::error::ErrorCode::Timeout);
    assert!(error
        .to_string()
        .contains("timed out waiting for networkidle"));
    handle.join().unwrap();
}
