use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use serde_json::json;

use crate::browser_cdp::render::{render_attached_page, BrowserAttachedPageRenderRequest};
use crate::error::ErrorCode;

use super::super::super::{read_cdp_request, reply_ok};

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
        locale: None,
        timezone_id: None,
        wait_for_selector: None,
        wait_for_images: false,
        scan_full_page: false,
        scroll_delay: Duration::ZERO,
        max_scroll_steps: 0,
        flatten_shadow_dom: false,
        process_iframes: false,
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
