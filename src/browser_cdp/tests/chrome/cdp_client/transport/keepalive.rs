use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;

use crate::browser_cdp::client::CdpClient;

use super::{mock_browser_ws_url, read_cdp_request, reply_ok};

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

    let mut client =
        CdpClient::connect(&mock_browser_ws_url(port), Duration::from_secs(2)).unwrap();
    client.set_keepalive_interval_for_test(Duration::from_millis(10));
    let result = client
        .send("Browser.getVersion", None, None, Duration::from_secs(2))
        .unwrap();
    assert_eq!(result["product"], "Chrome/mock-after-keepalive");
    handle.join().unwrap();
}
