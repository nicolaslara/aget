use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::session::Session;

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
