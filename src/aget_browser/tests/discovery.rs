use std::time::Duration;

use super::super::current_tab::CdpEndpointRequest;
use super::super::*;
use super::helpers::{http_json_response, serve_cdp_discovery_responses};

#[test]
fn discovers_cdp_endpoint_without_aget_facade_or_command_backend() {
    let list_body = r#"[
            {
                "type": "page",
                "webSocketDebuggerUrl": "ws://localhost:9999/devtools/page/ignored"
            },
            {
                "type": "browser",
                "webSocketDebuggerUrl": "ws://localhost:9999/devtools/browser/list"
            }
        ]"#;
    let (port, handle) = serve_cdp_discovery_responses(vec![
        http_json_response(200, r#"{}"#),
        http_json_response(200, list_body),
    ]);

    let endpoint = AgetBrowser::default()
        .discover_cdp_endpoint(CdpEndpointRequest {
            port,
            timeout: Duration::from_secs(2),
        })
        .unwrap();

    assert_eq!(
        endpoint.ws_url,
        format!("ws://127.0.0.1:{port}/devtools/browser/list")
    );
    handle.join().unwrap();
}
