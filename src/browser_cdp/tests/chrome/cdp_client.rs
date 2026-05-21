use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;
use tungstenite::Message;

use crate::error::ErrorCode;

use super::super::super::client::CdpClient;
use super::super::super::render::{render_attached_page, BrowserAttachedPageRenderRequest};
use super::super::super::*;
use super::{read_cdp_request, reply_ok};

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
fn browser_cdp_render_attached_page_reads_current_url_and_html_without_navigation() {
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
                        "url": "https://example.com/current",
                        "title": "Current"
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

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Runtime.evaluate");
        assert_eq!(request["sessionId"], "session-1");
        assert!(request["params"]["expression"]
            .as_str()
            .unwrap()
            .contains("querySelectorAll"));
        reply_ok(
            &mut websocket,
            &request,
            json!({ "result": { "type": "undefined" } }),
        );

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Runtime.evaluate");
        assert_eq!(request["sessionId"], "session-1");
        assert_eq!(request["params"]["expression"], "location.href");
        reply_ok(
            &mut websocket,
            &request,
            json!({ "result": { "type": "string", "value": "https://example.com/current" } }),
        );

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Runtime.evaluate");
        assert_eq!(request["sessionId"], "session-1");
        assert_eq!(
            request["params"]["expression"],
            "document.documentElement.outerHTML || ''"
        );
        reply_ok(
            &mut websocket,
            &request,
            json!({ "result": { "type": "string", "value": "<html><body><main>Current tab</main></body></html>" } }),
        );

        let _ = websocket.close(None);
    });

    let rendered = render_attached_page(BrowserAttachedPageRenderRequest {
        ws_url: &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        wait_for_selector: None,
        wait_for_images: false,
        flatten_shadow_dom: false,
        settle_delay: Duration::ZERO,
        page_timeout: Duration::from_secs(2),
        wait_for_timeout: None,
        timeout: Duration::from_secs(2),
    })
    .unwrap();

    assert_eq!(rendered.final_url, "https://example.com/current");
    assert_eq!(
        rendered.html,
        "<html><body><main>Current tab</main></body></html>"
    );
    assert!(rendered.warnings.is_empty());
    handle.join().unwrap();
}

#[test]
fn browser_cdp_render_attached_page_reports_no_page_targets() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Target.setDiscoverTargets");
        reply_ok(&mut websocket, &request, json!({}));

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Target.getTargets");
        reply_ok(&mut websocket, &request, json!({ "targetInfos": [] }));

        let _ = websocket.close(None);
    });

    let error = render_attached_page(BrowserAttachedPageRenderRequest {
        ws_url: &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        wait_for_selector: None,
        wait_for_images: false,
        flatten_shadow_dom: false,
        settle_delay: Duration::ZERO,
        page_timeout: Duration::from_secs(2),
        wait_for_timeout: None,
        timeout: Duration::from_secs(2),
    })
    .unwrap_err();

    assert_eq!(error.code(), ErrorCode::ExtractionFailed);
    assert!(error
        .to_string()
        .contains("owned browser current-tab CDP attach found no page targets"));
    handle.join().unwrap();
}
