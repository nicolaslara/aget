use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;

use crate::browser_cdp::client::CdpClient;

use super::{mock_browser_ws_url, read_cdp_request, reply_ok};

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

    let mut client =
        CdpClient::connect(&mock_browser_ws_url(port), Duration::from_secs(2)).unwrap();
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

    let mut client =
        CdpClient::connect(&mock_browser_ws_url(port), Duration::from_secs(2)).unwrap();
    let result = client
        .send("Browser.getVersion", None, None, Duration::from_secs(2))
        .unwrap();
    assert_eq!(result["product"], "Chrome/mock-after-prompt");
    handle.join().unwrap();
}
