mod cdp_client;
mod chrome_process;
mod real_chrome;

use std::net::TcpStream;

use serde_json::{json, Value};
use tungstenite::{Message, WebSocket};

fn read_cdp_request(websocket: &mut WebSocket<TcpStream>) -> Value {
    let Message::Text(text) = websocket.read().unwrap() else {
        panic!("expected text CDP command");
    };
    serde_json::from_str(text.as_ref()).unwrap()
}

fn reply_ok(websocket: &mut WebSocket<TcpStream>, request: &Value, result: Value) {
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

fn reply_ok_binary(websocket: &mut WebSocket<TcpStream>, request: &Value, result: Value) {
    let id = request
        .get("id")
        .and_then(serde_json::Value::as_u64)
        .unwrap();
    let bytes = json!({ "id": id, "result": result })
        .to_string()
        .into_bytes();
    websocket.send(Message::Binary(bytes.into())).unwrap();
}
