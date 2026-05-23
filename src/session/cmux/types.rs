use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmuxImportOptions {
    pub surface: String,
    pub name: String,
    pub domains: Vec<String>,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub(super) struct CmuxCookiesResponse {
    pub(super) cookies: Vec<CmuxCookie>,
}

#[derive(Debug, Deserialize)]
pub(super) struct CmuxCookie {
    pub(super) name: String,
    pub(super) value: String,
    pub(super) domain: String,
    pub(super) path: String,
    pub(super) secure: bool,
    pub(super) session_only: bool,
    pub(super) expires: Option<i64>,
}
