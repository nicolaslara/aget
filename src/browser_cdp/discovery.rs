use std::fs;
use std::net::TcpStream;
use std::path::Path;
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use url::Url;

use crate::error::{AgetError, ErrorCode};
use crate::process::TempOutputFile;

use super::client::CdpClient;
use super::{CHROME_SANDBOX_STARTUP_HINT, CHROME_SILENT_STARTUP_HINT};

pub(super) fn wait_for_devtools_active_port(
    child: &mut Child,
    user_data_dir: &Path,
    stderr_capture: &TempOutputFile,
    timeout: Duration,
) -> Result<String, AgetError> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(AgetError::Stable {
                code: ErrorCode::BackendUnavailable,
                message: format!(
                    "owned browser fallback Chrome exited before CDP startup with {status}"
                ),
            });
        }
        if let Some((port, path)) = read_devtools_active_port(user_data_dir) {
            return Ok(format!("ws://127.0.0.1:{port}{path}"));
        }
        if let Some(ws_url) = devtools_ws_url_from_chrome_stderr(stderr_capture) {
            return Ok(ws_url);
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(AgetError::Stable {
        code: ErrorCode::Timeout,
        message: "owned browser fallback timed out waiting for Chrome CDP startup".to_string(),
    })
}

fn devtools_ws_url_from_chrome_stderr(stderr_capture: &TempOutputFile) -> Option<String> {
    let stderr = stderr_capture.read_to_string().ok()?;
    devtools_ws_url_from_stderr(&stderr)
}

pub(super) fn devtools_ws_url_from_stderr(stderr: &str) -> Option<String> {
    stderr.lines().find_map(|line| {
        let rest = line.trim().strip_prefix("DevTools listening on ")?;
        let url = rest.split_whitespace().next().unwrap_or(rest);
        if url.starts_with("ws://") || url.starts_with("wss://") {
            Some(url.to_string())
        } else {
            None
        }
    })
}

pub(super) fn classify_chrome_startup_error(
    operation: &str,
    error: AgetError,
    stderr_capture: &TempOutputFile,
) -> AgetError {
    let stderr = stderr_capture.read_to_string().unwrap_or_default();
    let detail = relevant_chrome_stderr(&stderr);
    let combined = format!("{}\n{}", error, detail);
    if chrome_startup_requires_user_action(&combined) {
        return AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: if detail.is_empty() {
                format!("{operation} requires user action: {error}")
            } else {
                format!("{operation} requires user action: {error}; Chrome stderr: {detail}")
            },
        };
    }

    if detail.is_empty() {
        if stderr.trim().is_empty() {
            return append_silent_chrome_startup_hint(error);
        }
        return error;
    }

    match error {
        AgetError::Stable { code, message } => AgetError::Stable {
            code,
            message: format!("{message}; Chrome stderr: {detail}"),
        },
    }
}

fn chrome_startup_requires_user_action(output: &str) -> bool {
    let output = output.to_ascii_lowercase();
    [
        "opening in existing browser session",
        "profile lock",
        "profile is locked",
        "profile in use",
        "profile appears to be in use",
        "already running",
        "processsingleton",
        "singletonlock",
        "chrome must be quit",
        "quit chrome",
        "close chrome",
    ]
    .iter()
    .any(|needle| output.contains(needle))
}

pub(super) fn relevant_chrome_stderr(stderr: &str) -> String {
    let relevant = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            [
                "error",
                "fatal",
                "sandbox",
                "namespace",
                "permission",
                "cannot",
                "failed",
                "abort",
                "profile",
                "singleton",
                "already running",
                "opening in existing browser session",
            ]
            .iter()
            .any(|needle| lower.contains(needle))
        })
        .take(5)
        .collect::<Vec<_>>();
    if relevant.is_empty() {
        let lines = stderr
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n  ");
        if lines.is_empty() {
            String::new()
        } else {
            let count = lines.lines().count();
            format!("Chrome stderr (last {count} lines):\n  {lines}")
        }
    } else {
        append_chrome_startup_hint(relevant.join("\n  "))
    }
}

fn append_chrome_startup_hint(detail: String) -> String {
    let lower = detail.to_ascii_lowercase();
    if lower.contains("sandbox") || lower.contains("namespace") {
        format!("{detail}\n  {CHROME_SANDBOX_STARTUP_HINT}")
    } else {
        detail
    }
}

fn append_silent_chrome_startup_hint(error: AgetError) -> AgetError {
    match error {
        AgetError::Stable { code, message } => AgetError::Stable {
            code,
            message: format!("{message}; {CHROME_SILENT_STARTUP_HINT}"),
        },
    }
}

pub(super) fn read_devtools_active_port(user_data_dir: &Path) -> Option<(u16, String)> {
    let content = fs::read_to_string(user_data_dir.join("DevToolsActivePort")).ok()?;
    let mut lines = content.lines();
    let port = lines.next()?.trim().parse::<u16>().ok()?;
    let path = lines
        .next()
        .unwrap_or("/devtools/browser")
        .trim()
        .to_string();
    Some((port, path))
}

pub(super) fn connect_existing_profile_browser(
    profile_dir: &Path,
    timeout: Duration,
) -> Result<Option<CdpClient>, AgetError> {
    let Some((port, path)) = read_devtools_active_port(profile_dir) else {
        return Ok(None);
    };
    let ws_url = format!("ws://127.0.0.1:{port}{path}");
    match CdpClient::connect(&ws_url, timeout) {
        Ok(client) => Ok(Some(client)),
        Err(AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            ..
        }) => {
            let discovered_ws_url = match discover_cdp_ws_url(port, timeout) {
                Ok(ws_url) => ws_url,
                Err(_) => {
                    remove_stale_devtools_active_port(profile_dir);
                    return Ok(None);
                }
            };
            match CdpClient::connect(&discovered_ws_url, timeout) {
                Ok(client) => Ok(Some(client)),
                Err(AgetError::Stable {
                    code: ErrorCode::BackendUnavailable,
                    ..
                }) => {
                    remove_stale_devtools_active_port(profile_dir);
                    Ok(None)
                }
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

fn remove_stale_devtools_active_port(profile_dir: &Path) {
    let _ = fs::remove_file(profile_dir.join("DevToolsActivePort"));
}

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

pub(super) fn rewrite_cdp_ws_host(ws_url: &str, port: u16) -> Option<String> {
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

pub(super) fn wait_for_profile_browser_shutdown(profile_dir: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let Some((port, _)) = read_devtools_active_port(profile_dir) else {
            return;
        };
        if TcpStream::connect(("127.0.0.1", port)).is_err() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
}
