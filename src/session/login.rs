use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{AgetError, ErrorCode};
use crate::process::DEFAULT_SUBPROCESS_TIMEOUT;
use crate::session::agent_browser::{
    classify_agent_browser_failure, domain_allowed, domain_matches_allowed,
    filter_playwright_state, origin_host, read_filtered_agent_browser_session, run_agent_browser,
    set_private_file_permissions, AgentBrowserSessionFilter, RawStateFile,
};
use crate::session::{Session, SessionSource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginStartOptions {
    pub name: String,
    pub profile: Option<String>,
    pub url: String,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginFinishOptions {
    pub name: String,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginCancelOptions {
    pub name: String,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingLogin {
    pub name: String,
    pub profile: String,
    pub agent_session: String,
    pub url: String,
    pub allowed_domains: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_pid: Option<u32>,
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

pub fn start_login_session(options: LoginStartOptions) -> Result<LoginStartResult, AgetError> {
    validate_login_name(&options.name)?;
    let allowed_domains = allowed_domains_from_url(&options.url)?;
    let profile = options
        .profile
        .map(PathBuf::from)
        .unwrap_or_else(|| default_login_profile_path(&options.tmp_dir, &options.name));
    prepare_login_profile_path(&profile)?;
    let pending = PendingLogin {
        agent_session: format!("aget-login-{}", options.name),
        name: options.name,
        profile: profile.to_string_lossy().into_owned(),
        url: options.url,
        allowed_domains,
        browser_pid: None,
    };

    fs::create_dir_all(&options.tmp_dir).map_err(io_aget_error)?;
    write_pending_login(&options.tmp_dir, &pending)?;
    let open = run_agent_browser(
        &options.tmp_dir,
        &[
            "--profile",
            &pending.profile,
            "--session",
            &pending.agent_session,
            "open",
            &pending.url,
        ],
    )?;
    if !open.status.success() {
        let _ = remove_pending_login(&options.tmp_dir, &pending.name);
        return Err(classify_agent_browser_failure("open", &open));
    }
    Ok(LoginStartResult { pending })
}

pub(crate) fn start_owned_login_session(
    options: LoginStartOptions,
) -> Result<LoginStartResult, AgetError> {
    validate_login_name(&options.name)?;
    let allowed_domains = allowed_domains_from_url(&options.url)?;
    let profile = options
        .profile
        .map(PathBuf::from)
        .unwrap_or_else(|| default_owned_login_profile_path(&options.tmp_dir, &options.name));
    prepare_login_profile_path(&profile)?;
    let mut pending = PendingLogin {
        agent_session: format!("aget-login-{}", options.name),
        name: options.name,
        profile: profile.to_string_lossy().into_owned(),
        url: options.url,
        allowed_domains,
        browser_pid: None,
    };

    fs::create_dir_all(&options.tmp_dir).map_err(io_aget_error)?;
    write_pending_login(&options.tmp_dir, &pending)?;
    let started =
        crate::browser_cdp::start_login_browser(crate::browser_cdp::BrowserLoginStartRequest {
            profile_dir: &profile,
            url: &pending.url,
            timeout: DEFAULT_SUBPROCESS_TIMEOUT,
        });
    let started = match started {
        Ok(started) => started,
        Err(error) => {
            let _ = remove_pending_login(&options.tmp_dir, &pending.name);
            let _ = remove_tool_owned_login_profile(&options.tmp_dir, &pending);
            return Err(error);
        }
    };
    pending.browser_pid = Some(started.pid);
    if let Err(error) = rewrite_pending_login(&options.tmp_dir, &pending) {
        let _ = remove_pending_login(&options.tmp_dir, &pending.name);
        let _ = remove_tool_owned_login_profile(&options.tmp_dir, &pending);
        return Err(error);
    }
    Ok(LoginStartResult { pending })
}

pub fn finish_login_session(options: LoginFinishOptions) -> Result<LoginFinishResult, AgetError> {
    validate_login_name(&options.name)?;
    let pending = read_pending_login(&options.tmp_dir, &options.name)?;
    let raw_state =
        RawStateFile::new(&options.tmp_dir, "login-raw-state").map_err(io_aget_error)?;
    let raw_state_path = raw_state.path().to_string_lossy().into_owned();
    let save = run_agent_browser(
        &options.tmp_dir,
        &[
            "--session",
            &pending.agent_session,
            "state",
            "save",
            &raw_state_path,
        ],
    )?;
    if !save.status.success() {
        return Err(classify_agent_browser_failure("state save", &save));
    }
    if raw_state.path().exists() {
        set_private_file_permissions(raw_state.path()).map_err(io_aget_error)?;
    }
    let session = read_filtered_agent_browser_session(
        raw_state.path(),
        AgentBrowserSessionFilter {
            name: pending.name.clone(),
            source: SessionSource::AgentBrowser {
                session: pending.agent_session.clone(),
            },
            allowed_domains: pending.allowed_domains.clone(),
            source_session: pending.agent_session.clone(),
        },
    )?;
    if session.cookies.is_empty() && session.origins.is_empty() {
        return Err(AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: format!(
                "login flow exported no auth state for allowed domains: {}",
                pending.allowed_domains.join(", ")
            ),
        });
    }
    let close = run_agent_browser(
        &options.tmp_dir,
        &["--session", &pending.agent_session, "close"],
    )?;
    if !close.status.success() {
        return Err(classify_agent_browser_failure("close", &close));
    }
    Ok(LoginFinishResult { session, pending })
}

pub(crate) fn finish_owned_login_session(
    options: LoginFinishOptions,
) -> Result<LoginFinishResult, AgetError> {
    validate_login_name(&options.name)?;
    let pending = read_pending_login(&options.tmp_dir, &options.name)?;
    let state = crate::browser_cdp::export_login_browser_state(
        crate::browser_cdp::BrowserLoginStateExportRequest {
            profile_dir: Path::new(&pending.profile),
            allowed_domains: &pending.allowed_domains,
            pid: pending.browser_pid,
            timeout: DEFAULT_SUBPROCESS_TIMEOUT,
        },
    )?;
    let session = filter_playwright_state(
        state,
        AgentBrowserSessionFilter {
            name: pending.name.clone(),
            source: SessionSource::AgentBrowser {
                session: pending.agent_session.clone(),
            },
            allowed_domains: pending.allowed_domains.clone(),
            source_session: pending.agent_session.clone(),
        },
    )?;
    if session.cookies.is_empty() && session.origins.is_empty() {
        return Err(AgetError::Stable {
            code: ErrorCode::RequiresUserAction,
            message: format!(
                "owned login flow exported no auth state for allowed domains: {}",
                pending.allowed_domains.join(", ")
            ),
        });
    }
    Ok(LoginFinishResult { session, pending })
}

pub fn complete_login_session(options: LoginCompleteOptions) -> Result<(), AgetError> {
    validate_login_name(&options.pending.name)?;
    remove_tool_owned_login_profile(&options.tmp_dir, &options.pending).map_err(io_aget_error)?;
    remove_pending_login(&options.tmp_dir, &options.pending.name).map_err(io_aget_error)?;
    Ok(())
}

pub fn merge_login_session(mut existing: Session, mut fresh: Session) -> Session {
    let authorized_domains = fresh.allowed_cookie_domains.clone();
    let authorized_origins = fresh.allowed_storage_origins.clone();

    existing
        .cookies
        .retain(|cookie| !domain_allowed(&cookie.domain, &authorized_domains));
    existing.origins.retain(|origin| {
        !authorized_origins
            .iter()
            .any(|allowed| allowed == &origin.origin)
    });
    existing.allowed_cookie_domains.retain(|domain| {
        !authorized_domains
            .iter()
            .any(|allowed| domain_matches_allowed(domain, allowed))
    });
    existing
        .allowed_storage_origins
        .retain(|origin| !authorized_origins.iter().any(|allowed| allowed == origin));

    existing.source = fresh.source;
    existing.sensitive = existing.sensitive || fresh.sensitive;
    existing
        .allowed_cookie_domains
        .append(&mut fresh.allowed_cookie_domains);
    existing.allowed_cookie_domains.sort();
    existing.allowed_cookie_domains.dedup();
    existing
        .allowed_storage_origins
        .append(&mut fresh.allowed_storage_origins);
    existing.allowed_storage_origins.sort();
    existing.allowed_storage_origins.dedup();
    existing.cookies.append(&mut fresh.cookies);
    existing.origins.append(&mut fresh.origins);
    existing
}

pub fn cancel_login_session(options: LoginCancelOptions) -> Result<LoginCancelResult, AgetError> {
    validate_login_name(&options.name)?;
    let pending = read_pending_login(&options.tmp_dir, &options.name)?;
    let close = run_agent_browser(
        &options.tmp_dir,
        &["--session", &pending.agent_session, "close"],
    );
    let cleanup_result = (|| {
        remove_tool_owned_login_profile(&options.tmp_dir, &pending)?;
        remove_pending_login(&options.tmp_dir, &pending.name)
    })();
    match close {
        Ok(output) if output.status.success() => cleanup_result.map_err(io_aget_error)?,
        Ok(output) => {
            let cleanup_error = cleanup_result.err();
            let mut error = classify_agent_browser_failure("close", &output);
            if let (AgetError::Stable { message, .. }, Some(cleanup_error)) =
                (&mut error, cleanup_error)
            {
                message.push_str(&format!("; cleanup also failed: {cleanup_error}"));
            }
            return Err(error);
        }
        Err(error) => {
            cleanup_result.map_err(io_aget_error)?;
            return Err(error);
        }
    }
    Ok(LoginCancelResult { pending })
}

pub(crate) fn cancel_owned_login_session(
    options: LoginCancelOptions,
) -> Result<LoginCancelResult, AgetError> {
    validate_login_name(&options.name)?;
    let pending = read_pending_login(&options.tmp_dir, &options.name)?;
    let close =
        crate::browser_cdp::close_login_browser(crate::browser_cdp::BrowserLoginCloseRequest {
            profile_dir: Path::new(&pending.profile),
            pid: pending.browser_pid,
            timeout: DEFAULT_SUBPROCESS_TIMEOUT,
        });
    let cleanup_result = (|| {
        remove_tool_owned_login_profile(&options.tmp_dir, &pending)?;
        remove_pending_login(&options.tmp_dir, &pending.name)
    })();

    if let Err(error) = close {
        cleanup_result.map_err(io_aget_error)?;
        return Err(error);
    }
    cleanup_result.map_err(io_aget_error)?;
    Ok(LoginCancelResult { pending })
}

fn default_login_profile_path(tmp_dir: &Path, name: &str) -> PathBuf {
    tmp_dir.join("agent-browser").join(format!("aget-{name}"))
}

fn default_owned_login_profile_path(tmp_dir: &Path, name: &str) -> PathBuf {
    tmp_dir.join("owned-login").join(format!("aget-{name}"))
}

fn prepare_login_profile_path(profile: &Path) -> Result<(), AgetError> {
    if let Some(parent) = profile.parent() {
        fs::create_dir_all(parent).map_err(io_aget_error)?;
    }
    Ok(())
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

fn allowed_domains_from_url(url: &str) -> Result<Vec<String>, AgetError> {
    let Some((scheme, _)) = url.split_once("://") else {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("invalid login URL '{url}'"),
        });
    };
    if !scheme.eq_ignore_ascii_case("https") {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("login URL must use https, got '{url}'"),
        });
    }
    let Some(host) = origin_host(url) else {
        return Err(AgetError::Stable {
            code: ErrorCode::UsageError,
            message: format!("invalid login URL '{url}'"),
        });
    };

    if let Some(bare_host) = host.strip_prefix("www.") {
        Ok(vec![bare_host.to_string(), host])
    } else {
        Ok(vec![host])
    }
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

fn rewrite_pending_login(tmp_dir: &Path, pending: &PendingLogin) -> Result<(), AgetError> {
    let path = pending_login_path(tmp_dir, &pending.name);
    let mut options = OpenOptions::new();
    options.create(true).truncate(true).write(true);
    set_private_file_mode(&mut options);
    let mut file = options.open(&path).map_err(io_aget_error)?;
    serde_json::to_writer_pretty(&mut file, pending).map_err(|error| AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    })?;
    file.write_all(b"\n").map_err(io_aget_error)
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

fn remove_tool_owned_login_profile(tmp_dir: &Path, pending: &PendingLogin) -> io::Result<()> {
    let profile = PathBuf::from(&pending.profile);
    if profile != default_login_profile_path(tmp_dir, &pending.name)
        && profile != default_owned_login_profile_path(tmp_dir, &pending.name)
    {
        return Ok(());
    }

    remove_dir_all_with_retries(&profile)
}

fn remove_dir_all_with_retries(path: &Path) -> io::Result<()> {
    let mut last_error = None;
    for _ in 0..5 {
        match fs::remove_dir_all(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    Err(last_error.unwrap_or_else(|| io::Error::other("remove_dir_all failed")))
}

fn io_aget_error(error: impl std::fmt::Display) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}

#[cfg(unix)]
fn set_private_file_mode(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;

    options.mode(0o600);
}

#[cfg(not(unix))]
fn set_private_file_mode(_options: &mut OpenOptions) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(name: &str, profile: PathBuf) -> PendingLogin {
        PendingLogin {
            name: name.to_string(),
            profile: profile.to_string_lossy().into_owned(),
            agent_session: format!("aget-login-{name}"),
            url: "https://example.com/login".to_string(),
            allowed_domains: vec!["example.com".to_string()],
            browser_pid: None,
        }
    }

    #[test]
    fn complete_login_session_removes_owned_default_profile() {
        let temp = tempfile::tempdir().unwrap();
        let tmp_dir = temp.path().join("tmp");
        fs::create_dir_all(&tmp_dir).unwrap();
        let profile = default_owned_login_profile_path(&tmp_dir, "docs");
        fs::create_dir_all(&profile).unwrap();
        let pending = pending("docs", profile.clone());
        write_pending_login(&tmp_dir, &pending).unwrap();

        complete_login_session(LoginCompleteOptions {
            pending,
            tmp_dir: tmp_dir.clone(),
        })
        .unwrap();

        assert!(!profile.exists());
        assert!(!pending_login_path(&tmp_dir, "docs").exists());
    }

    #[test]
    fn complete_login_session_preserves_custom_profile_path() {
        let temp = tempfile::tempdir().unwrap();
        let tmp_dir = temp.path().join("tmp");
        let profile = temp.path().join("custom-profile");
        fs::create_dir_all(&tmp_dir).unwrap();
        fs::create_dir_all(&profile).unwrap();
        let pending = pending("docs", profile.clone());
        write_pending_login(&tmp_dir, &pending).unwrap();

        complete_login_session(LoginCompleteOptions {
            pending,
            tmp_dir: tmp_dir.clone(),
        })
        .unwrap();

        assert!(profile.exists());
        assert!(!pending_login_path(&tmp_dir, "docs").exists());
    }

    #[test]
    fn cancel_owned_login_session_cleans_pending_and_profile_when_browser_is_closed() {
        let temp = tempfile::tempdir().unwrap();
        let tmp_dir = temp.path().join("tmp");
        fs::create_dir_all(&tmp_dir).unwrap();
        let profile = default_owned_login_profile_path(&tmp_dir, "docs");
        fs::create_dir_all(&profile).unwrap();
        write_pending_login(&tmp_dir, &pending("docs", profile.clone())).unwrap();

        let result = cancel_owned_login_session(LoginCancelOptions {
            name: "docs".to_string(),
            tmp_dir: tmp_dir.clone(),
        })
        .unwrap();

        assert_eq!(result.pending.name, "docs");
        assert!(!profile.exists());
        assert!(!pending_login_path(&tmp_dir, "docs").exists());
    }

    #[test]
    fn start_owned_login_session_rejects_non_https_before_launching_browser() {
        let temp = tempfile::tempdir().unwrap();
        let tmp_dir = temp.path().join("tmp");

        let error = start_owned_login_session(LoginStartOptions {
            name: "docs".to_string(),
            profile: None,
            url: "http://example.com/login".to_string(),
            tmp_dir: tmp_dir.clone(),
        })
        .unwrap_err();

        assert_eq!(error.code(), ErrorCode::UsageError);
        assert!(!pending_login_path(&tmp_dir, "docs").exists());
    }
}
