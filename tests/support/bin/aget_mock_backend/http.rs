use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use serde_json::Value;

#[derive(Clone)]
pub(crate) struct ParsedUrl {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) path: String,
    pub(crate) origin: String,
}

pub(crate) struct HttpResponse {
    pub(crate) status: u16,
    pub(crate) final_url: String,
    pub(crate) headers: BTreeMap<String, String>,
    pub(crate) body: String,
}

pub(crate) fn session_headers(state: &Value, parsed: &ParsedUrl) -> BTreeMap<String, String> {
    let cookies = state["cookies"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|cookie| {
            let domain = cookie["domain"]
                .as_str()?
                .trim_start_matches('.')
                .to_lowercase();
            if parsed.host == domain || parsed.host.ends_with(&format!(".{domain}")) {
                Some(format!(
                    "{}={}",
                    cookie["name"].as_str()?,
                    cookie["value"].as_str()?
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut headers = BTreeMap::new();
    if !cookies.is_empty() {
        headers.insert("Cookie".to_string(), cookies.join("; "));
    }
    for origin in state["origins"].as_array().into_iter().flatten() {
        if origin["origin"].as_str() == Some(parsed.origin.as_str()) {
            for entry in origin["localStorage"].as_array().into_iter().flatten() {
                if entry["name"].as_str() == Some("local_token") {
                    if let Some(value) = entry["value"].as_str() {
                        headers.insert("X-Local-Token".to_string(), value.to_string());
                    }
                }
            }
        }
    }
    headers
}

pub(crate) fn fetch_following_redirects(
    url: &str,
    headers: &BTreeMap<String, String>,
) -> Result<HttpResponse, String> {
    let mut current = url.to_string();
    for _ in 0..5 {
        let response = fetch_once(&current, headers)?;
        if (300..400).contains(&response.status) {
            if let Some(location) = response.headers.get("location") {
                current = absolutize_url(&current, location)?;
                continue;
            }
        }
        return Ok(response);
    }
    Err("too many redirects".to_string())
}

fn fetch_once(url: &str, headers: &BTreeMap<String, String>) -> Result<HttpResponse, String> {
    let parsed = parse_http_url(url)?;
    let mut stream = TcpStream::connect((parsed.host.as_str(), parsed.port))
        .map_err(|error| format!("connect to {}:{}: {error}", parsed.host, parsed.port))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| format!("set read timeout: {error}"))?;

    write!(
        stream,
        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n",
        parsed.path, parsed.host, parsed.port
    )
    .map_err(|error| format!("write request: {error}"))?;
    for (name, value) in headers {
        write!(stream, "{name}: {value}\r\n").map_err(|error| format!("write header: {error}"))?;
    }
    write!(stream, "\r\n").map_err(|error| format!("finish request: {error}"))?;

    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|error| format!("read response: {error}"))?;
    parse_response(url, &raw)
}

fn parse_response(url: &str, raw: &[u8]) -> Result<HttpResponse, String> {
    let raw = String::from_utf8_lossy(raw);
    let (head, body) = raw
        .split_once("\r\n\r\n")
        .ok_or_else(|| "malformed HTTP response".to_string())?;
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .ok_or_else(|| "missing HTTP status".to_string())?;
    let mut headers = BTreeMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    let final_url = if (300..400).contains(&status) {
        headers
            .get("location")
            .map(|location| absolutize_url(url, location))
            .transpose()?
            .unwrap_or_else(|| url.to_string())
    } else {
        url.to_string()
    };
    Ok(HttpResponse {
        status,
        final_url,
        headers,
        body: body.to_string(),
    })
}

pub(crate) fn parse_http_url(url: &str) -> Result<ParsedUrl, String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("mock backend only supports http URLs, got {url}"))?;
    let (authority, path) = match rest.split_once('/') {
        Some((authority, path)) => (authority, format!("/{path}")),
        None => (rest, "/".to_string()),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (
            host.to_string(),
            port.parse::<u16>()
                .map_err(|error| format!("invalid port in {url}: {error}"))?,
        ),
        None => (authority.to_string(), 80),
    };
    Ok(ParsedUrl {
        origin: format!("http://{authority}"),
        host,
        port,
        path,
    })
}

fn absolutize_url(base: &str, location: &str) -> Result<String, String> {
    if location.starts_with("http://") {
        return Ok(location.to_string());
    }
    let parsed = parse_http_url(base)?;
    if location.starts_with('/') {
        Ok(format!("{}{}", parsed.origin, location))
    } else {
        let parent = parsed
            .path
            .rsplit_once('/')
            .map(|(parent, _)| parent)
            .unwrap_or("");
        Ok(format!("{}/{location}", parent.trim_end_matches('/')))
    }
}

pub(crate) fn element_body(html: &str, tag: &str) -> Option<String> {
    let start_marker = format!("<{tag}");
    let start = html.find(&start_marker)?;
    let body_start = html[start..].find('>')? + start + 1;
    let end = html[body_start..].find(&format!("</{tag}>"))? + body_start;
    Some(html[body_start..end].to_string())
}

pub(crate) fn remove_elements(html: &str, tag: &str) -> String {
    let mut output = html.to_string();
    while let Some(start) = output.find(&format!("<{tag}")) {
        let Some(after_open) = output[start..].find('>').map(|index| start + index + 1) else {
            break;
        };
        let Some(end) = output[after_open..]
            .find(&format!("</{tag}>"))
            .map(|index| after_open + index + tag.len() + 3)
        else {
            break;
        };
        output.replace_range(start..end, "");
    }
    output
}

pub(crate) fn textify(html: &str) -> String {
    let without_scripts = remove_elements(&remove_elements(html, "script"), "style");
    let mut text = String::new();
    let mut in_tag = false;
    for character in without_scripts.chars() {
        match character {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
