use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub fn local_server(body: &str) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let body = body.to_string();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let mut request = Vec::new();
        let mut buffer = [0; 512];
        while request.len() < 16 * 1024 {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    request.extend_from_slice(&buffer[..read]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
        let _ = stream.flush();
        let _ = stream.shutdown(Shutdown::Both);
    });
    (url, handle)
}

pub fn mock_current_tab_cdp_server(final_url: &str, html: &str) -> (u16, JoinHandle<()>) {
    mock_current_tab_cdp_server_with_screenshot(final_url, html, None)
}

pub fn mock_current_tab_cdp_server_with_screenshot(
    final_url: &str,
    html: &str,
    screenshot_base64: Option<&str>,
) -> (u16, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let final_url = final_url.to_string();
    let html = html.to_string();
    let screenshot_base64 = screenshot_base64.map(str::to_string);
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0; 512];
        let _ = stream.read(&mut request);
        let body =
            format!(r#"{{"webSocketDebuggerUrl":"ws://localhost:9999/devtools/browser/current"}}"#);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();

        let (stream, _) = listener.accept().unwrap();
        let mut websocket = tungstenite::accept(stream).unwrap();
        serve_attached_page_cdp(
            &mut websocket,
            &final_url,
            &html,
            screenshot_base64.as_deref(),
        );
        let _ = websocket.close(None);
    });
    (port, handle)
}

pub fn mock_current_tab_interact_cdp_server(
    final_url: &str,
    action_results: Vec<serde_json::Value>,
) -> (u16, JoinHandle<()>) {
    mock_current_tab_interact_cdp_server_with_page(
        final_url,
        "<html><body><main>Mock interact page</main></body></html>",
        None,
        action_results,
    )
}

pub fn mock_current_tab_interact_cdp_server_with_page(
    final_url: &str,
    html: &str,
    screenshot_base64: Option<&str>,
    action_results: Vec<serde_json::Value>,
) -> (u16, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let final_url = final_url.to_string();
    let html = html.to_string();
    let screenshot_base64 = screenshot_base64.map(str::to_string);
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0; 512];
        let _ = stream.read(&mut request);
        let body = r#"{"webSocketDebuggerUrl":"ws://localhost:9999/devtools/browser/current"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();

        let (stream, _) = listener.accept().unwrap();
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let mut websocket = tungstenite::accept(stream).unwrap();
        serve_interact_cdp(
            &mut websocket,
            &final_url,
            &html,
            screenshot_base64.as_deref(),
            action_results,
        );
        let _ = websocket.close(None);
    });
    (port, handle)
}

fn serve_attached_page_cdp(
    websocket: &mut tungstenite::WebSocket<std::net::TcpStream>,
    final_url: &str,
    html: &str,
    screenshot_base64: Option<&str>,
) {
    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.setDiscoverTargets");
    reply_ok(websocket, &request, serde_json::json!({}));

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.getTargets");
    reply_ok(
        websocket,
        &request,
        serde_json::json!({
            "targetInfos": [
                {
                    "targetId": "page-1",
                    "type": "page",
                    "url": final_url,
                    "title": "Current"
                }
            ]
        }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.attachToTarget");
    reply_ok(
        websocket,
        &request,
        serde_json::json!({ "sessionId": "session-1" }),
    );

    for expected_method in [
        "Page.enable",
        "Runtime.enable",
        "Runtime.runIfWaitingForDebugger",
        "Network.enable",
    ] {
        let request = read_cdp_request(websocket);
        assert_eq!(request["method"], expected_method);
        reply_ok(websocket, &request, serde_json::json!({}));
    }

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.setAutoAttach");
    reply_ok(websocket, &request, serde_json::json!({}));

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    reply_ok(
        websocket,
        &request,
        serde_json::json!({ "result": { "type": "undefined" } }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    assert_eq!(request["params"]["expression"], "location.href");
    reply_ok(
        websocket,
        &request,
        serde_json::json!({ "result": { "type": "string", "value": final_url } }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    assert_eq!(
        request["params"]["expression"],
        "document.documentElement.outerHTML || ''"
    );
    reply_ok(
        websocket,
        &request,
        serde_json::json!({ "result": { "type": "string", "value": html } }),
    );

    if let Some(data) = screenshot_base64 {
        let request = read_cdp_request(websocket);
        assert_eq!(request["method"], "Page.captureScreenshot");
        assert_eq!(request["params"]["format"], "png");
        reply_ok(websocket, &request, serde_json::json!({ "data": data }));
    }
}

fn serve_interact_cdp(
    websocket: &mut tungstenite::WebSocket<std::net::TcpStream>,
    final_url: &str,
    html: &str,
    screenshot_base64: Option<&str>,
    action_results: Vec<serde_json::Value>,
) {
    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.setDiscoverTargets");
    reply_ok(websocket, &request, serde_json::json!({}));

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.getTargets");
    reply_ok(
        websocket,
        &request,
        serde_json::json!({
            "targetInfos": [
                {
                    "targetId": "page-1",
                    "type": "page",
                    "url": final_url,
                    "title": "Current"
                }
            ]
        }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.attachToTarget");
    reply_ok(
        websocket,
        &request,
        serde_json::json!({ "sessionId": "session-1" }),
    );

    for expected_method in [
        "Page.enable",
        "Runtime.enable",
        "Runtime.runIfWaitingForDebugger",
        "Network.enable",
    ] {
        let request = read_cdp_request(websocket);
        assert_eq!(request["method"], expected_method);
        reply_ok(websocket, &request, serde_json::json!({}));
    }

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.setAutoAttach");
    reply_ok(websocket, &request, serde_json::json!({}));

    let mut action_results = action_results.into_iter();
    loop {
        let request = match read_optional_cdp_request(websocket) {
            Some(request) => request,
            None => break,
        };
        if request["method"] == "Page.captureScreenshot" {
            reply_ok(
                websocket,
                &request,
                serde_json::json!({ "data": screenshot_base64.unwrap_or("cGl4ZWxz") }),
            );
            continue;
        }
        assert_eq!(request["method"], "Runtime.evaluate");
        let expression = request["params"]["expression"].as_str().unwrap_or("");
        if expression == "location.href" {
            reply_ok(
                websocket,
                &request,
                serde_json::json!({ "result": { "type": "string", "value": final_url } }),
            );
        } else if expression == "document.readyState" {
            reply_ok(
                websocket,
                &request,
                serde_json::json!({ "result": { "type": "string", "value": "complete" } }),
            );
        } else if expression == "document.documentElement.outerHTML || ''" {
            reply_ok(
                websocket,
                &request,
                serde_json::json!({ "result": { "type": "string", "value": html } }),
            );
        } else if expression.contains("agetResolveUnique")
            || expression.contains("document.querySelectorAll")
            || expression.contains("document.querySelector(")
        {
            let result = action_results
                .next()
                .expect("unexpected extra action Runtime.evaluate call");
            reply_ok(
                websocket,
                &request,
                serde_json::json!({
                    "result": {
                        "type": "string",
                        "value": serde_json::to_string(&result).unwrap()
                    }
                }),
            );
        } else {
            panic!("unexpected Runtime.evaluate expression: {expression}");
        }
    }
}

fn read_cdp_request(
    websocket: &mut tungstenite::WebSocket<std::net::TcpStream>,
) -> serde_json::Value {
    let tungstenite::Message::Text(text) = websocket.read().unwrap() else {
        panic!("expected text CDP command");
    };
    serde_json::from_str(text.as_ref()).unwrap()
}

fn read_optional_cdp_request(
    websocket: &mut tungstenite::WebSocket<std::net::TcpStream>,
) -> Option<serde_json::Value> {
    let Ok(message) = websocket.read() else {
        return None;
    };
    let tungstenite::Message::Text(text) = message else {
        return None;
    };
    Some(serde_json::from_str(text.as_ref()).unwrap())
}

fn reply_ok(
    websocket: &mut tungstenite::WebSocket<std::net::TcpStream>,
    request: &serde_json::Value,
    result: serde_json::Value,
) {
    let id = request
        .get("id")
        .and_then(serde_json::Value::as_u64)
        .unwrap();
    websocket
        .send(tungstenite::Message::Text(
            serde_json::json!({ "id": id, "result": result })
                .to_string()
                .into(),
        ))
        .unwrap();
}
