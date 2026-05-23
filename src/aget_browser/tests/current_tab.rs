use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use super::super::current_tab::CurrentTabRequest;
use super::super::*;
use super::helpers::{http_json_response, serve_attached_page_cdp};

#[test]
fn renders_current_tab_from_explicit_cdp_port_without_aget_facade_or_command_backend() {
    use std::io::{Read as _, Write as _};

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0u8; 512];
        let _ = stream.read(&mut request);
        let body = r#"{"webSocketDebuggerUrl":"ws://localhost:9999/devtools/browser/current"}"#;
        stream
            .write_all(http_json_response(200, body).as_bytes())
            .unwrap();

        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        serve_attached_page_cdp(
            &mut websocket,
            "https://example.com/current",
            "<html><body>Current composed page</body></html>",
        );
        let _ = websocket.close(None);
    });

    let rendered = AgetBrowser::default()
        .render_current_tab(CurrentTabRequest {
            port,
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
            discovery_timeout: Duration::from_secs(2),
            page_timeout: Duration::from_secs(2),
            wait_for_timeout: None,
            timeout: Duration::from_secs(2),
        })
        .unwrap();

    assert_eq!(
        rendered.cdp_ws_url,
        format!("ws://127.0.0.1:{port}/devtools/browser/current")
    );
    assert_eq!(rendered.final_url, "https://example.com/current");
    assert_eq!(
        rendered.html,
        "<html><body>Current composed page</body></html>"
    );
    assert!(rendered.warnings.is_empty());
    handle.join().unwrap();
}
