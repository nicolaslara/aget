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
