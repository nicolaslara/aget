use std::collections::BTreeMap;
use std::io::Read;
use std::net::TcpStream;

use super::MockRequest;

pub(super) fn read_request(stream: &mut TcpStream) -> Option<MockRequest> {
    let mut buffer = Vec::new();
    let mut chunk = [0u8; 1024];
    while buffer.len() < 16 * 1024 {
        let read = stream.read(&mut chunk).ok()?;
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    if buffer.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(&buffer);
    let mut lines = text.lines();
    let request_line = lines.next()?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next()?.to_string();
    let raw_path = request_parts.next()?.to_string();
    let path = raw_path
        .split('?')
        .next()
        .unwrap_or(raw_path.as_str())
        .to_string();
    let mut headers = BTreeMap::new();
    for line in lines {
        if line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    Some(MockRequest {
        method,
        path,
        headers,
    })
}

pub(super) fn has_cookie(request: &MockRequest, name: &str, value: &str) -> bool {
    request
        .headers
        .get("cookie")
        .is_some_and(|cookie| cookie_contains(cookie, name, value))
}

pub(super) fn cookie_contains(cookie_header: &str, name: &str, value: &str) -> bool {
    cookie_header
        .split(';')
        .map(str::trim)
        .any(|cookie| cookie == format!("{name}={value}"))
}

pub(super) fn html(status: u16, headers: &[(&str, &str)], body: &str) -> String {
    http_response(status, "text/html; charset=utf-8", headers, body)
}

pub(super) fn http_response(
    status: u16,
    content_type: &str,
    headers: &[(&str, &str)],
    body: &str,
) -> String {
    let reason = match status {
        200 => "OK",
        302 => "Found",
        401 => "Unauthorized",
        404 => "Not Found",
        _ => "OK",
    };
    let mut response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n",
        body.len()
    );
    for (name, value) in headers {
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str("\r\n");
    response.push_str(body);
    response
}
