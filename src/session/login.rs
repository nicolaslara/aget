use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AgetError, ErrorCode};
use crate::session::agent_browser::{
    classify_agent_browser_failure, domain_allowed, domain_matches_allowed, origin_host,
    read_filtered_agent_browser_session, run_agent_browser, set_private_file_permissions,
    AgentBrowserSessionFilter, RawStateFile,
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
    validate_login_name(&options.name)?;
    let pending = read_pending_login(&options.tmp_dir, &options.name)?;
    let raw_state =
        RawStateFile::new(&options.tmp_dir, "login-raw-state").map_err(io_aget_error)?;
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
    let close = run_agent_browser(&["--session", &pending.agent_session, "close"])?;
    if !close.status.success() {
        return Err(classify_agent_browser_failure("close", &close));
    }
    Ok(LoginFinishResult { session, pending })
}

pub fn complete_login_session(options: LoginCompleteOptions) -> Result<(), AgetError> {
    validate_login_name(&options.pending.name)?;
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
    let close = run_agent_browser(&["--session", &pending.agent_session, "close"])?;
    if !close.status.success() {
        return Err(classify_agent_browser_failure("close", &close));
    }
    remove_pending_login(&options.tmp_dir, &pending.name).map_err(io_aget_error)?;
    Ok(LoginCancelResult { pending })
}

fn default_login_profile_path(tmp_dir: &Path, name: &str) -> PathBuf {
    tmp_dir.join("agent-browser").join(format!("aget-{name}"))
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
