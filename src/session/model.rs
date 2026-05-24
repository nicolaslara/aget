use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub version: u32,
    pub name: String,
    pub source: SessionSource,
    pub sensitive: bool,
    pub allowed_cookie_domains: Vec<String>,
    pub allowed_storage_origins: Vec<String>,
    pub cookies: Vec<SessionCookie>,
    pub origins: Vec<SessionOrigin>,
}

impl Session {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            version: 1,
            name: name.into(),
            source: SessionSource::Manual,
            sensitive: true,
            allowed_cookie_domains: Vec::new(),
            allowed_storage_origins: Vec::new(),
            cookies: Vec::new(),
            origins: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SessionSource {
    Manual,
    Cmux { surface: String },
    ChromeProfile { profile: String },
    BrowserLogin { session: String },
    Composed { sessions: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<i64>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: Option<String>,
    pub source_session: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionOrigin {
    pub origin: String,
    pub local_storage: Vec<StorageEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub session_storage: Vec<StorageEntry>,
    pub source_session: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageEntry {
    pub name: String,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_defaults_to_sensitive_v1_manual() {
        let session = Session::new("example");

        assert_eq!(session.version, 1);
        assert_eq!(session.name, "example");
        assert!(session.sensitive);
        assert_eq!(session.source, SessionSource::Manual);
    }
}
