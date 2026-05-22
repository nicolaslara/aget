use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;

use crate::browser_cdp::client::CdpClient;

use super::super::{read_cdp_request, reply_ok, reply_ok_binary};

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

#[test]
fn direct_page_cdp_connection_enables_domains_without_session_ids() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        for expected_method in [
            "Page.enable",
            "Runtime.enable",
            "Runtime.runIfWaitingForDebugger",
            "Network.enable",
        ] {
            let request = read_cdp_request(&mut websocket);
            assert_eq!(request["method"], expected_method);
            assert!(request.get("sessionId").is_none());
            reply_ok(&mut websocket, &request, json!({}));
        }
        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/page/direct"),
        Duration::from_secs(2),
    )
    .unwrap();
    let page = client.attach_existing_page(Duration::from_secs(2)).unwrap();
    let page = page.expect("direct page connection should expose current page");
    assert!(page.target_id.is_empty());
    assert!(page.session_id.is_empty());
    client
        .enable_page_domains(&page.session_id, Duration::from_secs(2))
        .unwrap();
    handle.join().unwrap();
}

#[test]
fn browser_cdp_attach_existing_page_enables_discovery_before_target_list() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Target.setDiscoverTargets");
        assert_eq!(request["params"]["discover"], true);
        assert!(request.get("sessionId").is_none());
        reply_ok(&mut websocket, &request, json!({}));

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Target.getTargets");
        assert!(request.get("sessionId").is_none());
        reply_ok(
            &mut websocket,
            &request,
            json!({
                "targetInfos": [
                    {
                        "targetId": "page-1",
                        "type": "page",
                        "url": "https://example.com/",
                        "title": "Example"
                    }
                ]
            }),
        );

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Target.attachToTarget");
        assert_eq!(request["params"]["targetId"], "page-1");
        assert_eq!(request["params"]["flatten"], true);
        assert!(request.get("sessionId").is_none());
        reply_ok(
            &mut websocket,
            &request,
            json!({ "sessionId": "session-1" }),
        );
        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    let page = client
        .attach_existing_page(Duration::from_secs(2))
        .unwrap()
        .expect("existing page should attach");
    assert_eq!(page.target_id, "page-1");
    assert_eq!(page.session_id, "session-1");
    handle.join().unwrap();
}

#[test]
fn browser_cdp_enable_page_domains_auto_attaches_subtargets() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        for expected_method in [
            "Page.enable",
            "Runtime.enable",
            "Runtime.runIfWaitingForDebugger",
            "Network.enable",
        ] {
            let request = read_cdp_request(&mut websocket);
            assert_eq!(request["method"], expected_method);
            assert_eq!(request["sessionId"], "session-1");
            reply_ok(&mut websocket, &request, json!({}));
        }

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Target.setAutoAttach");
        assert_eq!(request["sessionId"], "session-1");
        assert_eq!(request["params"]["autoAttach"], true);
        assert_eq!(request["params"]["waitForDebuggerOnStart"], false);
        assert_eq!(request["params"]["flatten"], true);
        reply_ok(&mut websocket, &request, json!({}));
        let _ = websocket.close(None);
    });

    let mut client = CdpClient::connect(
        &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        Duration::from_secs(2),
    )
    .unwrap();
    client
        .enable_page_domains("session-1", Duration::from_secs(2))
        .unwrap();
    handle.join().unwrap();
}
