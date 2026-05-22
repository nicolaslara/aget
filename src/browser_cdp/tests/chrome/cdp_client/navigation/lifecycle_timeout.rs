use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;

use crate::browser_cdp::client::CdpClient;
use crate::browser_cdp::PageWaitUntil;

use super::super::super::{read_cdp_request, reply_ok};

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
