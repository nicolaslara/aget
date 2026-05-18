use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use crate::error::{AgetError, ErrorCode};
use crate::process::{
    configure_local_command, create_private_file as create_private_output_file, wait_for_child,
    TempOutputFile, DEFAULT_SUBPROCESS_TIMEOUT,
};
use crate::session::{Session, SessionCookie, SessionOrigin, SessionSource, StorageEntry};

#[derive(Debug, Deserialize)]
pub(crate) struct AgentBrowserState {
    #[serde(default)]
    cookies: Vec<AgentBrowserCookie>,
    #[serde(default)]
    origins: Vec<AgentBrowserOrigin>,
}

#[derive(Debug, Deserialize)]
struct AgentBrowserCookie {
    name: String,
    value: String,
    domain: String,
    path: String,
    #[serde(default, deserialize_with = "deserialize_agent_browser_expires")]
    expires: Option<i64>,
    #[serde(rename = "httpOnly", default)]
    http_only: bool,
    #[serde(default)]
    secure: bool,
    #[serde(rename = "sameSite", default)]
    same_site: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AgentBrowserOrigin {
    origin: String,
    #[serde(rename = "localStorage", default)]
    local_storage: Vec<StorageEntry>,
    #[serde(rename = "sessionStorage", default)]
    _session_storage: Vec<StorageEntry>,
}

fn deserialize_agent_browser_expires<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let expires = Option::<serde_json::Number>::deserialize(deserializer)?;
    expires
        .map(|number| {
            number
                .as_i64()
                .or_else(|| number.as_f64().map(|value| value.trunc() as i64))
                .ok_or_else(|| Error::custom("agent-browser expires must be numeric"))
        })
        .transpose()
}

#[derive(Debug)]
pub(crate) struct AgentBrowserOutput {
    pub(crate) status: std::process::ExitStatus,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

#[derive(Debug, Clone)]
pub(crate) struct AgentBrowserSessionFilter {
    pub(crate) name: String,
    pub(crate) source: SessionSource,
    pub(crate) allowed_domains: Vec<String>,
    pub(crate) source_session: String,
}

pub(crate) fn run_agent_browser(
    tmp_dir: &Path,
    args: &[&str],
) -> Result<AgentBrowserOutput, AgetError> {
    let command =
        env::var("AGET_AGENT_BROWSER_COMMAND").unwrap_or_else(|_| "agent-browser".to_string());
    let stdout_file =
        TempOutputFile::new(tmp_dir, "agent-browser-stdout").map_err(io_aget_error)?;
    let stderr_file =
        TempOutputFile::new(tmp_dir, "agent-browser-stderr").map_err(io_aget_error)?;
    let stdout = create_private_output_file(stdout_file.path()).map_err(io_aget_error)?;
    let stderr = create_private_output_file(stderr_file.path()).map_err(io_aget_error)?;
    let mut command_builder = Command::new(&command);
    command_builder
        .args(args)
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    configure_local_command(&mut command_builder);
    let mut child = command_builder
        .spawn()
        .map_err(|error| backend_unavailable(&command, error))?;
    let status = wait_for_child(&mut child, DEFAULT_SUBPROCESS_TIMEOUT, || {
        format!(
            "agent-browser timed out after {} seconds",
            DEFAULT_SUBPROCESS_TIMEOUT.as_secs()
        )
    })?;
    Ok(AgentBrowserOutput {
        status,
        stdout: stdout_file.read_to_string().map_err(io_aget_error)?,
        stderr: stderr_file.read_to_string().map_err(io_aget_error)?,
    })
}

pub(crate) fn classify_agent_browser_failure(
    action: &str,
    output: &AgentBrowserOutput,
) -> AgetError {
    let combined = format!("{}\n{}", output.stdout, output.stderr);
    if indicates_user_action(&combined) {
        return AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: user_action_message(action, combined.trim()),
        };
    }

    let code = if matches!(output.status.code(), Some(126 | 127)) {
        ErrorCode::BackendUnavailable
    } else {
        ErrorCode::ExtractionFailed
    };
    AgetError::Stable {
        code,
        message: if combined.trim().is_empty() {
            format!("agent-browser {action} exited with {}", output.status)
        } else {
            combined.trim().to_string()
        },
    }
}

fn indicates_user_action(output: &str) -> bool {
    let output = output.to_ascii_lowercase();
    [
        "quit chrome",
        "close chrome",
        "chrome must be quit",
        "profile lock",
        "profile is locked",
        "profile in use",
        "already running",
        "login needed",
        "please log in",
        "login required",
        "not logged in",
        "please sign in",
        "sign in required",
        "no auth state",
        "no authentication state",
    ]
    .iter()
    .any(|needle| output.contains(needle))
}

fn user_action_message(action: &str, details: &str) -> String {
    if details.is_empty() {
        format!("agent-browser {action} requires user action")
    } else {
        format!("agent-browser {action} requires user action: {details}")
    }
}

pub(crate) fn read_filtered_agent_browser_session(
    raw_state_path: &Path,
    filter: AgentBrowserSessionFilter,
) -> Result<Session, AgetError> {
    let file = File::open(raw_state_path).map_err(|error| AgetError::Stable {
        code: ErrorCode::ExtractionFailed,
        message: format!("agent-browser did not write raw state: {error}"),
    })?;
    let state: AgentBrowserState =
        serde_json::from_reader(file).map_err(|error| AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: format!("agent-browser returned malformed state JSON: {error}"),
        })?;

    filter_agent_browser_state(state, filter)
}

pub(crate) fn filter_agent_browser_state(
    state: AgentBrowserState,
    filter: AgentBrowserSessionFilter,
) -> Result<Session, AgetError> {
    let mut cookies_by_key = BTreeMap::new();
    for cookie in state.cookies {
        if !domain_allowed(&cookie.domain, &filter.allowed_domains) {
            continue;
        }

        let session_cookie = SessionCookie {
            name: cookie.name,
            value: cookie.value,
            domain: cookie.domain,
            path: cookie.path,
            expires: cookie.expires,
            http_only: cookie.http_only,
            secure: cookie.secure,
            same_site: cookie.same_site,
            source_session: Some(filter.source_session.clone()),
        };
        let key = (
            session_cookie.name.clone(),
            normalize_domain(&session_cookie.domain),
            session_cookie.path.clone(),
        );
        match cookies_by_key.get(&key) {
            Some(existing) if existing != &session_cookie => {
                return Err(AgetError::Stable {
                    code: ErrorCode::SessionConflict,
                    message: format!(
                        "agent-browser returned conflicting duplicate cookie '{}' for domain '{}' and path '{}'",
                        key.0, key.1, key.2
                    ),
                });
            }
            Some(_) => {}
            None => {
                cookies_by_key.insert(key, session_cookie);
            }
        }
    }

    let mut origins_by_name = BTreeMap::new();
    for origin in state.origins {
        let Some(host) = origin_host(&origin.origin) else {
            continue;
        };
        if !domain_allowed(&host, &filter.allowed_domains) {
            continue;
        }

        let session_origin = SessionOrigin {
            origin: origin.origin,
            local_storage: origin.local_storage,
            source_session: Some(filter.source_session.clone()),
        };
        match origins_by_name.get(&session_origin.origin) {
            Some(existing) if existing != &session_origin => {
                return Err(AgetError::Stable {
                    code: ErrorCode::SessionConflict,
                    message: format!(
                        "agent-browser returned conflicting duplicate storage origin '{}'",
                        session_origin.origin
                    ),
                });
            }
            Some(_) => {}
            None => {
                origins_by_name.insert(session_origin.origin.clone(), session_origin);
            }
        }
    }

    let mut session = Session::new(filter.name);
    session.source = filter.source;
    session.sensitive = true;
    session.allowed_cookie_domains = filter.allowed_domains;
    session.cookies = cookies_by_key.into_values().collect();
    session.origins = origins_by_name.into_values().collect();
    session.allowed_storage_origins = session
        .origins
        .iter()
        .map(|origin| origin.origin.clone())
        .collect();
    Ok(session)
}

pub(crate) fn domain_allowed(candidate_domain: &str, allowed_domains: &[String]) -> bool {
    allowed_domains
        .iter()
        .any(|allowed| domain_matches_allowed(candidate_domain, allowed))
}

pub(crate) fn domain_matches_allowed(candidate_domain: &str, allowed_domain: &str) -> bool {
    let candidate = normalize_domain(candidate_domain);
    let allowed = normalize_domain(allowed_domain);
    if candidate.is_empty() || allowed.is_empty() {
        return false;
    }

    if candidate == allowed {
        return true;
    }

    if let Some(candidate_root) = candidate.strip_prefix('.') {
        return candidate_root == allowed;
    }

    candidate.ends_with(&format!(".{allowed}"))
}

pub(crate) fn origin_host(origin: &str) -> Option<String> {
    let (_, rest) = origin.split_once("://")?;
    let authority = rest.split('/').next().unwrap_or(rest);
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = if let Some(stripped) = host_port.strip_prefix('[') {
        stripped.split(']').next()?
    } else {
        host_port.split(':').next().unwrap_or(host_port)
    };
    let host = normalize_domain(host);
    (!host.is_empty()).then_some(host)
}

pub(crate) fn normalize_domain(domain: &str) -> String {
    domain
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

pub(crate) struct RawStateFile {
    path: PathBuf,
}

impl RawStateFile {
    pub(crate) fn new(dir: &Path, prefix: &str) -> io::Result<Self> {
        let path = unique_raw_state_path(dir, prefix);
        drop(create_private_file(&path, true)?);
        Ok(Self { path })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for RawStateFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn unique_raw_state_path(dir: &Path, prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    dir.join(format!("{prefix}-{}-{nanos}.json", std::process::id()))
}

fn create_private_file(path: &Path, create_new: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true);
    if create_new {
        options.create_new(true);
    } else {
        options.create(true).truncate(true);
    }
    set_private_file_mode(&mut options);
    let file = options.open(path)?;
    set_private_file_permissions(path)?;
    Ok(file)
}

#[cfg(unix)]
fn set_private_file_mode(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;

    options.mode(0o600);
}

#[cfg(not(unix))]
fn set_private_file_mode(_options: &mut OpenOptions) {}

#[cfg(unix)]
pub(crate) fn set_private_file_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
pub(crate) fn set_private_file_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn backend_unavailable(command: &str, error: io::Error) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("agent-browser backend is unavailable for command '{command}': {error}"),
    }
}

fn io_aget_error(error: impl std::fmt::Display) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_cookies_and_origins_by_allowed_domains() {
        let state = AgentBrowserState {
            cookies: vec![
                cookie("sid", "example.com"),
                cookie("sub", "docs.example.com"),
                cookie("evil", "example.com.evil"),
            ],
            origins: vec![
                origin("https://example.com"),
                origin("https://docs.example.com:443"),
                origin("https://example.com.evil"),
            ],
        };

        let session = filter_agent_browser_state(state, filter()).unwrap();

        assert_eq!(session.cookies.len(), 2);
        assert!(session.cookies.iter().any(|cookie| cookie.name == "sid"));
        assert!(session.cookies.iter().any(|cookie| cookie.name == "sub"));
        assert!(!session.cookies.iter().any(|cookie| cookie.name == "evil"));
        assert_eq!(session.origins.len(), 2);
        assert!(session
            .origins
            .iter()
            .any(|origin| origin.origin == "https://example.com"));
        assert!(session
            .origins
            .iter()
            .any(|origin| origin.origin == "https://docs.example.com:443"));
        assert_eq!(
            session.source,
            SessionSource::ChromeProfile {
                profile: "Default".to_string()
            }
        );
        assert!(session.sensitive);
    }

    #[test]
    fn rejects_conflicting_duplicate_cookies() {
        let mut duplicate = cookie("sid", "example.com");
        duplicate.value = "different-secret".to_string();
        let state = AgentBrowserState {
            cookies: vec![cookie("sid", "example.com"), duplicate],
            origins: Vec::new(),
        };

        let error = filter_agent_browser_state(state, filter()).unwrap_err();

        assert_eq!(error.code(), ErrorCode::SessionConflict);
    }

    #[test]
    fn rejects_conflicting_duplicate_origins() {
        let mut duplicate = origin("https://example.com");
        duplicate.local_storage = vec![StorageEntry {
            name: "token".to_string(),
            value: "different-secret".to_string(),
        }];
        let state = AgentBrowserState {
            cookies: Vec::new(),
            origins: vec![origin("https://example.com"), duplicate],
        };

        let error = filter_agent_browser_state(state, filter()).unwrap_err();

        assert_eq!(error.code(), ErrorCode::SessionConflict);
    }

    #[cfg(unix)]
    #[test]
    fn raw_state_file_uses_private_permissions_and_is_removed_on_drop() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let raw_state = RawStateFile::new(temp.path(), "agent-browser-raw-state").unwrap();
        let path = raw_state.path().to_path_buf();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;

        assert_eq!(mode, 0o600);
        drop(raw_state);
        assert!(!path.exists());
    }

    #[test]
    fn origin_host_parses_http_hosts() {
        assert_eq!(
            origin_host("https://example.com:443"),
            Some("example.com".to_string())
        );
        assert_eq!(
            origin_host("https://user@example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(origin_host("file:///tmp/page.html"), None);
    }

    #[test]
    fn user_action_detection_matches_profile_and_login_failures() {
        assert!(indicates_user_action(
            "Please quit Chrome before using this profile"
        ));
        assert!(indicates_user_action("login needed for this site"));
        assert!(!indicates_user_action("syntax error"));
    }

    fn filter() -> AgentBrowserSessionFilter {
        AgentBrowserSessionFilter {
            name: "demo".to_string(),
            source: SessionSource::ChromeProfile {
                profile: "Default".to_string(),
            },
            allowed_domains: vec!["example.com".to_string()],
            source_session: "aget-import-test".to_string(),
        }
    }

    fn cookie(name: &str, domain: &str) -> AgentBrowserCookie {
        AgentBrowserCookie {
            name: name.to_string(),
            value: format!("{name}-secret"),
            domain: domain.to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: true,
            secure: true,
            same_site: Some("Lax".to_string()),
        }
    }

    fn origin(origin: &str) -> AgentBrowserOrigin {
        AgentBrowserOrigin {
            origin: origin.to_string(),
            local_storage: vec![StorageEntry {
                name: "token".to_string(),
                value: "secret".to_string(),
            }],
            _session_storage: Vec::new(),
        }
    }
}
