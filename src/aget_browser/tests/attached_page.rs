use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use super::super::current_tab::AttachedPageRequest;
use super::super::*;
use super::helpers::serve_attached_page_cdp;

#[test]
fn renders_attached_page_without_aget_facade_or_command_backend() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        serve_attached_page_cdp(
            &mut websocket,
            "https://example.com/current",
            "<html><body>Current engine page</body></html>",
        );
        let _ = websocket.close(None);
    });

    let rendered = AgetBrowser::default()
        .render_attached_page(AttachedPageRequest {
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
        .unwrap();

    assert_eq!(rendered.final_url, "https://example.com/current");
    assert_eq!(
        rendered.html,
        "<html><body>Current engine page</body></html>"
    );
    assert!(rendered.warnings.is_empty());
    handle.join().unwrap();
}
