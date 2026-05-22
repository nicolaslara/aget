use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;
use tungstenite::Message;

use crate::browser_cdp::client::CdpClient;
use crate::browser_cdp::PageWaitUntil;

use super::super::super::super::{read_cdp_request, reply_ok};

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
