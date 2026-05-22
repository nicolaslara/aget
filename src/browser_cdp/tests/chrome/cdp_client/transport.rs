use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;

use crate::browser_cdp::client::{cdp_websocket_config, CdpClient};

use super::super::{read_cdp_request, reply_ok, reply_ok_binary};

#[test]
fn browser_cdp_websocket_config_allows_large_frames() {
    let config = cdp_websocket_config();

    assert_eq!(config.max_message_size, None);
    assert_eq!(config.max_frame_size, None);
}

#[test]
fn browser_cdp_sends_keepalive_ping_while_waiting_for_response() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Browser.getVersion");

        loop {
            match websocket.read().unwrap() {
                tungstenite::Message::Ping(_) => {
                    reply_ok(
                        &mut websocket,
                        &request,
                        json!({ "product": "Chrome/mock-after-keepalive" }),
                    );
                    let _ = websocket.close(None);
                    break;
                }
                tungstenite::Message::Pong(_)
                | tungstenite::Message::Text(_)
                | tungstenite::Message::Binary(_)
                | tungstenite::Message::Frame(_) => {}
                tungstenite::Message::Close(_) => panic!("client closed before keepalive ping"),
            }
        }
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    client.set_keepalive_interval_for_test(Duration::from_millis(10));
    let result = client
        .send("Browser.getVersion", None, None, Duration::from_secs(2))
        .unwrap();
    assert_eq!(result["product"], "Chrome/mock-after-keepalive");
    handle.join().unwrap();
}

#[test]
fn browser_cdp_auto_accepts_alert_dialogs_while_waiting_for_response() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Browser.getVersion");
        websocket
            .send(tungstenite::Message::Text(
                json!({
                    "method": "Page.javascriptDialogOpening",
                    "sessionId": "session-1",
                    "params": {
                        "type": "alert",
                        "message": "Hello",
                    }
                })
                .to_string()
                .into(),
            ))
            .unwrap();

        let dialog_request = read_cdp_request(&mut websocket);
        assert_eq!(dialog_request["method"], "Page.handleJavaScriptDialog");
        assert_eq!(dialog_request["sessionId"], "session-1");
        assert_eq!(dialog_request["params"]["accept"], true);
        reply_ok(&mut websocket, &dialog_request, json!({}));
        reply_ok(
            &mut websocket,
            &request,
            json!({ "product": "Chrome/mock-after-alert" }),
        );
        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    let result = client
        .send("Browser.getVersion", None, None, Duration::from_secs(2))
        .unwrap();
    assert_eq!(result["product"], "Chrome/mock-after-alert");
    handle.join().unwrap();
}

#[test]
fn browser_cdp_does_not_auto_accept_prompt_dialogs() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_millis(250)))
            .unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Browser.getVersion");
        websocket
            .send(tungstenite::Message::Text(
                json!({
                    "method": "Page.javascriptDialogOpening",
                    "sessionId": "session-1",
                    "params": {
                        "type": "prompt",
                        "message": "Name?",
                    }
                })
                .to_string()
                .into(),
            ))
            .unwrap();

        match websocket.read() {
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            other => panic!("prompt dialog should not be auto-handled, got {other:?}"),
        }
        reply_ok(
            &mut websocket,
            &request,
            json!({ "product": "Chrome/mock-after-prompt" }),
        );
        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    let result = client
        .send("Browser.getVersion", None, None, Duration::from_secs(2))
        .unwrap();
    assert_eq!(result["product"], "Chrome/mock-after-prompt");
    handle.join().unwrap();
}

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

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
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

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
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

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    let result = client
        .send("Browser.getVersion", None, None, Duration::from_secs(2))
        .unwrap();
    assert_eq!(result["product"], "Chrome/mock-after-malformed-frames");
    handle.join().unwrap();
}
