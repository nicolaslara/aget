mod compose;
mod state_file;

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::session::StorageEntry;

pub use self::compose::{compose_playwright_state, compose_session};
pub use self::state_file::TempStateFile;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaywrightState {
    pub cookies: Vec<PlaywrightCookie>,
    pub origins: Vec<PlaywrightOrigin>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlaywrightCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<i64>,
    #[serde(rename = "httpOnly")]
    pub http_only: bool,
    pub secure: bool,
    #[serde(rename = "sameSite", skip_serializing_if = "Option::is_none")]
    pub same_site: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaywrightOrigin {
    pub origin: String,
    #[serde(rename = "localStorage")]
    pub local_storage: Vec<StorageEntry>,
    #[serde(
        rename = "sessionStorage",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub session_storage: Vec<StorageEntry>,
}
