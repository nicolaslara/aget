use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;

use crate::browser_cdp::render::{render_attached_page, BrowserAttachedPageRenderRequest};

use super::super::super::{read_cdp_request, reply_ok};

#[test]
fn browser_cdp_render_attached_page_scans_full_page_before_capture() {
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
        reply_ok(
            &mut websocket,
            &request,
            json!({
                "targetInfos": [{
                    "targetId": "page-1",
                    "type": "page",
                    "url": "https://example.com/current",
                    "title": "Current"
                }]
            }),
        );

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Target.attachToTarget");
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
        reply_ok(&mut websocket, &request, json!({}));

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Runtime.evaluate");
        assert_eq!(request["sessionId"], "session-1");
        assert_eq!(request["params"]["awaitPromise"], true);
        let expression = request["params"]["expression"].as_str().unwrap();
        assert!(expression.contains("const delayMs = 150"));
        assert!(expression.contains("const maxSteps = 3"));
        assert!(expression.contains("window.scrollTo(0, totalHeight)"));
        reply_ok(
            &mut websocket,
            &request,
            json!({ "result": { "type": "object", "value": { "steps": 3, "totalHeight": 2400 } } }),
        );

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Runtime.evaluate");
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
        assert_eq!(request["params"]["expression"], "location.href");
        reply_ok(
            &mut websocket,
            &request,
            json!({ "result": { "type": "string", "value": "https://example.com/current" } }),
        );

        let request = read_cdp_request(&mut websocket);
        assert_eq!(request["method"], "Runtime.evaluate");
        assert_eq!(
            request["params"]["expression"],
            "document.documentElement.outerHTML || ''"
        );
        reply_ok(
            &mut websocket,
            &request,
            json!({ "result": { "type": "string", "value": "<html><body><main>Scanned</main></body></html>" } }),
        );

        let _ = websocket.close(None);
    });

    let rendered = render_attached_page(BrowserAttachedPageRenderRequest {
        ws_url: &format!("ws://127.0.0.1:{port}/devtools/browser/mock"),
        locale: None,
        timezone_id: None,
        wait_for_selector: None,
        wait_for_images: false,
        scan_full_page: true,
        scroll_delay: Duration::from_millis(150),
        max_scroll_steps: 3,
        flatten_shadow_dom: false,
        process_iframes: false,
        settle_delay: Duration::ZERO,
        page_timeout: Duration::from_secs(2),
        wait_for_timeout: None,
        timeout: Duration::from_secs(2),
    })
    .unwrap();

    assert_eq!(rendered.final_url, "https://example.com/current");
    assert_eq!(
        rendered.html,
        "<html><body><main>Scanned</main></body></html>"
    );
    assert!(rendered.warnings.is_empty());
    handle.join().unwrap();
}
