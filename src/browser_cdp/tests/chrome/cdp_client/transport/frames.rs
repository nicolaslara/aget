use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;

use crate::browser_cdp::client::CdpClient;

use super::{mock_browser_ws_url, read_cdp_request, reply_ok, reply_ok_binary};

#[test]
fn browser_cdp_accepts_binary_response_frames() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Browser.getVersion");
        reply_ok_binary(
            &mut websocket,
            &request,
            json!({ "product": "Chrome/mock-binary" }),
        );
        let _ = websocket.close(None);
    });

    let mut client =
        CdpClient::connect(&mock_browser_ws_url(port), Duration::from_secs(2)).unwrap();
    let result = client
        .send("Browser.getVersion", None, None, Duration::from_secs(2))
        .unwrap();
    assert_eq!(result["product"], "Chrome/mock-binary");
    handle.join().unwrap();
}

#[test]
fn browser_cdp_skips_invalid_binary_response_frames() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Browser.getVersion");
        websocket
            .send(tungstenite::Message::Binary(vec![0xff, 0xfe].into()))
            .unwrap();
        reply_ok(
            &mut websocket,
            &request,
            json!({ "product": "Chrome/mock-after-invalid-binary" }),
        );
        let _ = websocket.close(None);
    });

    let mut client =
        CdpClient::connect(&mock_browser_ws_url(port), Duration::from_secs(2)).unwrap();
    let result = client
        .send("Browser.getVersion", None, None, Duration::from_secs(2))
        .unwrap();
    assert_eq!(result["product"], "Chrome/mock-after-invalid-binary");
    handle.join().unwrap();
}

#[test]
fn browser_cdp_skips_malformed_response_frames() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Browser.getVersion");
        websocket
            .send(tungstenite::Message::Text("{not-cdp-json".into()))
            .unwrap();
        websocket
            .send(tungstenite::Message::Binary(
                b"{also-not-cdp-json".to_vec().into(),
            ))
            .unwrap();
        reply_ok(
            &mut websocket,
            &request,
            json!({ "product": "Chrome/mock-after-malformed-frames" }),
        );
        let _ = websocket.close(None);
    });

    let mut client =
        CdpClient::connect(&mock_browser_ws_url(port), Duration::from_secs(2)).unwrap();
    let result = client
        .send("Browser.getVersion", None, None, Duration::from_secs(2))
        .unwrap();
    assert_eq!(result["product"], "Chrome/mock-after-malformed-frames");
    handle.join().unwrap();
}
