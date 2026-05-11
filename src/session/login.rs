use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{AgetError, ErrorCode};
use crate::session::{Session, SessionCookie, SessionOrigin, SessionSource, StorageEntry};

const HELLOINTERVIEW: &str = "hellointerview";
const HELLOINTERVIEW_DOMAINS: [&str; 2] = ["hellointerview.com", "www.hellointerview.com"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginStartOptions {
    pub site: String,
    pub name: Option<String>,
    pub profile: Option<String>,
    pub url: String,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginFinishOptions {
    pub site: String,
    pub name: Option<String>,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginCancelOptions {
    pub site: String,
    pub name: Option<String>,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingLogin {
    pub site: String,
    pub name: String,
    pub profile: String,
    pub agent_session: String,
    pub url: String,
    pub allowed_domains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoginStartResult {
    pub pending: PendingLogin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoginFinishResult {
    pub session: Session,
    pub pending: PendingLogin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginCompleteOptions {
    pub pending: PendingLogin,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoginCancelResult {
    pub pending: PendingLogin,
}

#[derive(Debug, Deserialize)]
struct AgentBrowserState {
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
struct AgentBrowserOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

pub fn start_login_session(options: LoginStartOptions) -> Result<LoginStartResult, AgetError> {
    let site = login_site(&options.site)?;
    let name = options
        .name
        .unwrap_or_else(|| site.default_name.to_string());
    validate_login_name(&name)?;
    let profile = options
        .profile
        .map(PathBuf::from)
        .unwrap_or_else(|| default_login_profile_path(&options.tmp_dir, site.default_name));
    validate_login_url_host(&options.url, site.domains)?;
    prepare_login_profile_path(&profile)?;
    let pending = PendingLogin {
        site: site.name.to_string(),
        agent_session: format!("aget-login-{name}"),
        name,
        profile: profile.to_string_lossy().into_owned(),
        url: options.url,
        allowed_domains: site
            .domains
            .iter()
            .map(|domain| domain.to_string())
            .collect(),
    };

    fs::create_dir_all(&options.tmp_dir).map_err(io_aget_error)?;
    write_pending_login(&options.tmp_dir, &pending)?;
    let open = run_agent_browser(&[
        "--profile",
        &pending.profile,
        "--session",
        &pending.agent_session,
        "open",
        &pending.url,
    ])?;
    if !open.status.success() {
        let _ = remove_pending_login(&options.tmp_dir, &pending.name);
        return Err(classify_agent_browser_failure("open", &open));
    }
    Ok(LoginStartResult { pending })
}

pub fn finish_login_session(options: LoginFinishOptions) -> Result<LoginFinishResult, AgetError> {
    let site = login_site(&options.site)?;
    let name = options
        .name
        .unwrap_or_else(|| site.default_name.to_string());
    validate_login_name(&name)?;
    let pending = read_pending_login(&options.tmp_dir, &name)?;
    ensure_pending_site(&pending, site.name)?;
    let raw_state = RawStateFile::new(&options.tmp_dir).map_err(io_aget_error)?;
    let raw_state_path = raw_state.path().to_string_lossy().into_owned();
    let save = run_agent_browser(&[
        "--session",
        &pending.agent_session,
        "state",
        "save",
        &raw_state_path,
    ])?;
    if !save.status.success() {
        return Err(classify_agent_browser_failure("state save", &save));
    }
    if raw_state.path().exists() {
        set_private_file_permissions(raw_state.path()).map_err(io_aget_error)?;
    }
    let session = read_filtered_session(raw_state.path(), &pending)?;
    if session.cookies.is_empty() && session.origins.is_empty() {
        return Err(AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: format!(
                "login flow exported no auth state for allowed domains: {}",
                pending.allowed_domains.join(", ")
            ),
        });
    }
    let close = run_agent_browser(&["--session", &pending.agent_session, "close"])?;
    if !close.status.success() {
        return Err(classify_agent_browser_failure("close", &close));
    }
    Ok(LoginFinishResult { session, pending })
}

pub fn complete_login_session(options: LoginCompleteOptions) -> Result<(), AgetError> {
    let site = login_site(&options.pending.site)?;
    validate_login_name(&options.pending.name)?;
    ensure_pending_site(&options.pending, site.name)?;
    remove_pending_login(&options.tmp_dir, &options.pending.name).map_err(io_aget_error)?;
    Ok(())
}

pub fn cancel_login_session(options: LoginCancelOptions) -> Result<LoginCancelResult, AgetError> {
    let site = login_site(&options.site)?;
    let name = options
        .name
        .unwrap_or_else(|| site.default_name.to_string());
    validate_login_name(&name)?;
    let pending = read_pending_login(&options.tmp_dir, &name)?;
    ensure_pending_site(&pending, site.name)?;
    let close = run_agent_browser(&["--session", &pending.agent_session, "close"])?;
    if !close.status.success() {
        return Err(classify_agent_browser_failure("close", &close));
    }
    remove_pending_login(&options.tmp_dir, &name).map_err(io_aget_error)?;
    Ok(LoginCancelResult { pending })
}

fn read_filtered_session(path: &Path, pending: &PendingLogin) -> Result<Session, AgetError> {
    let file = File::open(path).map_err(|error| AgetError::Stable {
        code: ErrorCode::ExtractionFailed,
        message: format!("agent-browser did not write raw state: {error}"),
    })?;
    let state: AgentBrowserState =
        serde_json::from_reader(file).map_err(|error| AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: format!("agent-browser returned malformed state JSON: {error}"),
        })?;
    filter_agent_browser_state(state, pending)
}

fn filter_agent_browser_state(
    state: AgentBrowserState,
    pending: &PendingLogin,
) -> Result<Session, AgetError> {
    let mut cookies_by_key = BTreeMap::new();
    for cookie in state.cookies {
        if !domain_allowed(&cookie.domain, &pending.allowed_domains) {
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
            source_session: Some(pending.agent_session.clone()),
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
        if !domain_allowed(&host, &pending.allowed_domains) {
            continue;
        }
        let session_origin = SessionOrigin {
            origin: origin.origin,
            local_storage: origin.local_storage,
            source_session: Some(pending.agent_session.clone()),
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

    let mut session = Session::new(&pending.name);
    session.source = SessionSource::AgentBrowser {
        session: pending.agent_session.clone(),
    };
    session.sensitive = true;
    session.allowed_cookie_domains = pending.allowed_domains.clone();
    session.cookies = cookies_by_key.into_values().collect();
    session.origins = origins_by_name.into_values().collect();
    session.allowed_storage_origins = session
        .origins
        .iter()
        .map(|origin| origin.origin.clone())
        .collect();
    Ok(session)
}

struct LoginSite {
    name: &'static str,
    default_name: &'static str,
    domains: &'static [&'static str],
}

fn login_site(site: &str) -> Result<LoginSite, AgetError> {
    match site {
        HELLOINTERVIEW => Ok(LoginSite {
            name: HELLOINTERVIEW,
            default_name: HELLOINTERVIEW,
            domains: &HELLOINTERVIEW_DOMAINS,
        }),
        _ => Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("unsupported login site '{site}'"),
        }),
    }
}

fn default_login_profile_path(tmp_dir: &Path, site_default_name: &str) -> PathBuf {
    tmp_dir
        .join("agent-browser")
        .join(format!("aget-{site_default_name}"))
}

fn prepare_login_profile_path(profile: &Path) -> Result<(), AgetError> {
    if let Some(parent) = profile.parent() {
        fs::create_dir_all(parent).map_err(io_aget_error)?;
    }
    Ok(())
}

fn ensure_pending_site(pending: &PendingLogin, site: &str) -> Result<(), AgetError> {
    if pending.site == site {
        Ok(())
    } else {
        Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!(
                "pending login '{}' belongs to site '{}'",
                pending.name, pending.site
            ),
        })
    }
}

fn pending_login_path(tmp_dir: &Path, name: &str) -> PathBuf {
    tmp_dir.join(format!("login-{name}.json"))
}

fn validate_login_name(name: &str) -> Result<(), AgetError> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains(':')
    {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("invalid login session name '{name}'"),
        });
    }
    Ok(())
}

fn validate_login_url_host(url: &str, allowed_domains: &[&str]) -> Result<(), AgetError> {
    let Some((scheme, _)) = url.split_once("://") else {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("invalid HelloInterview login URL '{url}'"),
        });
    };
    if !scheme.eq_ignore_ascii_case("https") {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!(
                "HelloInterview login URL must use https://hellointerview.com or https://www.hellointerview.com, got '{url}'"
            ),
        });
    }
    let Some(host) = origin_host(url) else {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("invalid HelloInterview login URL '{url}'"),
        });
    };

    let host_allowed = allowed_domains
        .iter()
        .map(|allowed| normalize_domain(allowed))
        .any(|allowed| host == allowed);
    if host_allowed {
        return Ok(());
    }

    Err(AgetError::Stable {
        code: ErrorCode::UsageError,
        message: format!(
            "HelloInterview login URL host '{host}' must be one of: {}",
            allowed_domains.join(", ")
        ),
    })
}

fn write_pending_login(tmp_dir: &Path, pending: &PendingLogin) -> Result<(), AgetError> {
    let path = pending_login_path(tmp_dir, &pending.name);
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    set_private_file_mode(&mut options);
    let mut file = match options.open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Err(AgetError::Stable {
                code: ErrorCode::UsageError,
                message: format!("pending login flow named '{}' already exists", pending.name),
            });
        }
        Err(error) => return Err(io_aget_error(error)),
    };
    let write_result = (|| {
        serde_json::to_writer_pretty(&mut file, pending).map_err(|error| AgetError::Stable {
            code: ErrorCode::IoError,
            message: error.to_string(),
        })?;
        file.write_all(b"\n").map_err(io_aget_error)
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_file(&path);
        return Err(error);
    }
    Ok(())
}

fn read_pending_login(tmp_dir: &Path, name: &str) -> Result<PendingLogin, AgetError> {
    let file =
        File::open(pending_login_path(tmp_dir, name)).map_err(|error| AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: format!("no pending login flow named '{name}': {error}"),
        })?;
    serde_json::from_reader(file).map_err(|error| AgetError::Stable {
        code: ErrorCode::ExtractionFailed,
        message: format!("pending login metadata is malformed: {error}"),
    })
}

fn remove_pending_login(tmp_dir: &Path, name: &str) -> io::Result<()> {
    match fs::remove_file(pending_login_path(tmp_dir, name)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn run_agent_browser(args: &[&str]) -> Result<AgentBrowserOutput, AgetError> {
    let command =
        env::var("AGET_AGENT_BROWSER_COMMAND").unwrap_or_else(|_| "agent-browser".to_string());
    let output = Command::new(&command)
        .args(args)
        .output()
        .map_err(|error| backend_unavailable(&command, error))?;
    Ok(AgentBrowserOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn classify_agent_browser_failure(action: &str, output: &AgentBrowserOutput) -> AgetError {
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
        "profile lock",
        "profile is locked",
        "profile in use",
        "already running",
        "login needed",
        "log in",
        "not logged in",
        "sign in",
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

fn domain_allowed(candidate_domain: &str, allowed_domains: &[String]) -> bool {
    allowed_domains
        .iter()
        .any(|allowed| domain_matches_allowed(candidate_domain, allowed))
}

fn domain_matches_allowed(candidate_domain: &str, allowed_domain: &str) -> bool {
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

fn origin_host(origin: &str) -> Option<String> {
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

fn normalize_domain(domain: &str) -> String {
    domain
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase()
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

struct RawStateFile {
    path: PathBuf,
}

impl RawStateFile {
    fn new(dir: &Path) -> io::Result<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = dir.join(format!(
            "login-raw-state-{}-{nanos}.json",
            std::process::id()
        ));
        drop(create_private_file(&path)?);
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for RawStateFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn create_private_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
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
fn set_private_file_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}
