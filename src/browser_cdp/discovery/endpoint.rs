use std::time::Duration;

use serde_json::Value;
use url::Url;

use crate::browser_cdp::client::CdpClient;
use crate::error::{AgetError, ErrorCode};

pub(crate) fn discover_cdp_ws_url(port: u16, timeout: Duration) -> Result<String, AgetError> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout.min(Duration::from_secs(2))))
        .http_status_as_error(false)
        .build();
    let agent: ureq::Agent = config.into();

    let version_error = match discover_cdp_ws_url_from_version(&agent, port) {
        Ok(ws_url) => return Ok(ws_url),
        Err(error) => error,
    };
    match discover_cdp_ws_url_from_list(&agent, port) {
        Ok(ws_url) => Ok(ws_url),
        Err(list_error) => match discover_cdp_ws_url_from_direct_ws(port, timeout) {
            Ok(ws_url) => Ok(ws_url),
            Err(ws_error) => Err(AgetError::Stable {
                code: ErrorCode::BackendUnavailable,
                message: format!(
                    "owned browser CDP discovery failed: /json/version: {version_error}; /json/list: {list_error}; /devtools/browser: {ws_error}",
                ),
            }),
        },
    }
}

fn discover_cdp_ws_url_from_version(agent: &ureq::Agent, port: u16) -> Result<String, AgetError> {
    let body = fetch_cdp_discovery_body(agent, port, "/json/version")?;
    let json: Value = serde_json::from_str(&body).map_err(|error| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!(
            "owned browser could not parse Chrome CDP /json/version response: {error}"
        ),
    })?;
    let ws_url = json
        .get("webSocketDebuggerUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: "owned browser CDP /json/version response lacked webSocketDebuggerUrl"
                .to_string(),
        })?;
    rewrite_cdp_ws_host(ws_url, port).ok_or_else(|| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("owned browser CDP discovery returned invalid WebSocket URL: {ws_url}"),
    })
}

fn discover_cdp_ws_url_from_list(agent: &ureq::Agent, port: u16) -> Result<String, AgetError> {
    let body = fetch_cdp_discovery_body(agent, port, "/json/list")?;
    let targets: Vec<Value> = serde_json::from_str(&body).map_err(|error| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("owned browser could not parse Chrome CDP /json/list response: {error}"),
    })?;
    let target = targets
        .iter()
        .find(|target| {
            target.get("type").and_then(Value::as_str) == Some("browser")
                && target
                    .get("webSocketDebuggerUrl")
                    .and_then(Value::as_str)
                    .is_some()
        })
        .or_else(|| {
            targets.iter().find(|target| {
                target
                    .get("webSocketDebuggerUrl")
                    .and_then(Value::as_str)
                    .is_some()
            })
        });
    let ws_url = target
        .and_then(|target| target.get("webSocketDebuggerUrl"))
        .and_then(Value::as_str)
        .ok_or_else(|| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: "owned browser CDP /json/list response lacked webSocketDebuggerUrl"
                .to_string(),
        })?;
    rewrite_cdp_ws_host(ws_url, port).ok_or_else(|| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("owned browser CDP discovery returned invalid WebSocket URL: {ws_url}"),
    })
}

fn fetch_cdp_discovery_body(
    agent: &ureq::Agent,
    port: u16,
    path: &str,
) -> Result<String, AgetError> {
    let url = format!("http://127.0.0.1:{port}{path}");
    let mut response = agent.get(&url).call().map_err(cdp_discovery_error)?;
    let status = response.status();
    if !status.is_success() {
        return Err(AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!("owned browser CDP discovery {path} returned HTTP {status}"),
        });
    }
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(cdp_discovery_error)?;
    Ok(body)
}

fn discover_cdp_ws_url_from_direct_ws(port: u16, timeout: Duration) -> Result<String, AgetError> {
    let ws_url = format!("ws://127.0.0.1:{port}/devtools/browser");
    let mut client = CdpClient::connect(&ws_url, timeout).map_err(|error| AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("owned browser direct CDP WebSocket discovery failed: {error}"),
    })?;
    client
        .send("Browser.getVersion", None, None, timeout)
        .map_err(|error| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!("owned browser direct CDP WebSocket verification failed: {error}"),
        })?;
    Ok(ws_url)
}

pub(in crate::browser_cdp) fn rewrite_cdp_ws_host(ws_url: &str, port: u16) -> Option<String> {
    let mut url = Url::parse(ws_url).ok()?;
    match url.scheme() {
        "ws" | "wss" => {}
        _ => return None,
    }
    url.set_host(Some("127.0.0.1")).ok()?;
    url.set_port(Some(port)).ok()?;
    Some(url.to_string())
}

fn cdp_discovery_error(error: ureq::Error) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("owned browser CDP discovery failed: {error}"),
    }
}
