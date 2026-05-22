use std::net::{TcpListener, TcpStream};
use std::thread;

use serde_json::{json, Value};
use tungstenite::{Message, WebSocket};

pub(super) fn read_cdp_request(websocket: &mut WebSocket<TcpStream>) -> Value {
    let Message::Text(text) = websocket.read().unwrap() else {
        panic!("expected text CDP command");
    };
    serde_json::from_str(text.as_ref()).unwrap()
}

pub(super) fn reply_ok(websocket: &mut WebSocket<TcpStream>, request: &Value, result: Value) {
    let id = request
        .get("id")
        .and_then(serde_json::Value::as_u64)
        .unwrap();
    websocket
        .send(Message::Text(
            json!({ "id": id, "result": result }).to_string().into(),
        ))
        .unwrap();
}

pub(super) fn serve_cdp_discovery_responses(
    responses: Vec<String>,
) -> (u16, thread::JoinHandle<()>) {
    use std::io::{Read as _, Write as _};

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 512];
            let _ = stream.read(&mut request);
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (port, handle)
}

pub(super) fn http_json_response(status: u16, body: &str) -> String {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Status",
    };
    format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )
}

pub(super) fn serve_attached_page_cdp(
    websocket: &mut WebSocket<TcpStream>,
    final_url: &str,
    html: &str,
) {
    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.setDiscoverTargets");
    reply_ok(websocket, &request, json!({}));

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.getTargets");
    reply_ok(
        websocket,
        &request,
        json!({
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
    assert_eq!(request["params"]["targetId"], "page-1");
    reply_ok(websocket, &request, json!({ "sessionId": "session-1" }));

    for expected_method in [
        "Page.enable",
        "Runtime.enable",
        "Runtime.runIfWaitingForDebugger",
        "Network.enable",
    ] {
        let request = read_cdp_request(websocket);
        assert_eq!(request["method"], expected_method);
        assert_eq!(request["sessionId"], "session-1");
        reply_ok(websocket, &request, json!({}));
    }

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Target.setAutoAttach");
    assert_eq!(request["sessionId"], "session-1");
    reply_ok(websocket, &request, json!({}));

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    assert_eq!(request["sessionId"], "session-1");
    assert!(request["params"]["expression"]
        .as_str()
        .unwrap()
        .contains("querySelectorAll"));
    reply_ok(
        websocket,
        &request,
        json!({ "result": { "type": "undefined" } }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    assert_eq!(request["sessionId"], "session-1");
    assert_eq!(request["params"]["expression"], "location.href");
    reply_ok(
        websocket,
        &request,
        json!({ "result": { "type": "string", "value": final_url } }),
    );

    let request = read_cdp_request(websocket);
    assert_eq!(request["method"], "Runtime.evaluate");
    assert_eq!(request["sessionId"], "session-1");
    assert_eq!(
        request["params"]["expression"],
        "document.documentElement.outerHTML || ''"
    );
    reply_ok(
        websocket,
        &request,
        json!({ "result": { "type": "string", "value": html } }),
    );
}
