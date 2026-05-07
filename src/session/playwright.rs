use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{AgetError, ErrorCode};
use crate::session::{Session, StorageEntry};

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
}

#[derive(Debug)]
pub struct TempStateFile {
    path: PathBuf,
}

impl TempStateFile {
    pub fn write(dir: &Path, state: &PlaywrightState) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let path = unique_state_path(dir);
        let mut file = create_private_file(&path)?;
        serde_json::to_writer_pretty(&mut file, state).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempStateFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn compose_playwright_state(sessions: &[Session]) -> Result<PlaywrightState, AgetError> {
    let mut cookies_by_key = BTreeMap::<CookieKey, PlaywrightCookie>::new();
    let mut origins_by_name = BTreeMap::<String, PlaywrightOrigin>::new();

    for session in sessions {
        for cookie in &session.cookies {
            let playwright_cookie = PlaywrightCookie {
                name: cookie.name.clone(),
                value: cookie.value.clone(),
                domain: cookie.domain.clone(),
                path: cookie.path.clone(),
                expires: cookie.expires,
                http_only: cookie.http_only,
                secure: cookie.secure,
                same_site: cookie.same_site.clone(),
            };
            let key = CookieKey::from(&playwright_cookie);

            match cookies_by_key.get(&key) {
                Some(existing) if existing != &playwright_cookie => {
                    return Err(AgetError::Stable {
                        code: ErrorCode::SessionConflict,
                        message: format!(
                            "conflicting cookie '{}' for domain '{}' and path '{}'",
                            key.name, key.domain, key.path
                        ),
                    });
                }
                Some(_) => {}
                None => {
                    cookies_by_key.insert(key, playwright_cookie);
                }
            }
        }

        for origin in &session.origins {
            let playwright_origin = PlaywrightOrigin {
                origin: origin.origin.clone(),
                local_storage: origin.local_storage.clone(),
            };
            match origins_by_name.get(&origin.origin) {
                Some(existing) if existing != &playwright_origin => {
                    return Err(AgetError::Stable {
                        code: ErrorCode::SessionConflict,
                        message: format!("conflicting storage origin '{}'", origin.origin),
                    });
                }
                Some(_) => {}
                None => {
                    origins_by_name.insert(origin.origin.clone(), playwright_origin);
                }
            }
        }
    }

    Ok(PlaywrightState {
        cookies: cookies_by_key.into_values().collect(),
        origins: origins_by_name.into_values().collect(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CookieKey {
    name: String,
    domain: String,
    path: String,
}

impl From<&PlaywrightCookie> for CookieKey {
    fn from(cookie: &PlaywrightCookie) -> Self {
        Self {
            name: cookie.name.clone(),
            domain: cookie.domain.clone(),
            path: cookie.path.clone(),
        }
    }
}

fn unique_state_path(dir: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    dir.join(format!(
        "playwright-state-{}-{nanos}.json",
        std::process::id()
    ))
}

fn create_private_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{SessionCookie, SessionOrigin};

    #[test]
    fn empty_sessions_produce_empty_state() {
        let state = compose_playwright_state(&[]).unwrap();

        assert!(state.cookies.is_empty());
        assert!(state.origins.is_empty());
    }

    #[test]
    fn maps_one_session_to_playwright_state() {
        let session = session_with_cookie("demo", "sid", "secret");
        let state = compose_playwright_state(&[session]).unwrap();

        assert_eq!(state.cookies.len(), 1);
        assert_eq!(state.cookies[0].name, "sid");
        assert_eq!(state.cookies[0].value, "secret");
    }

    #[test]
    fn deduplicates_identical_cookies() {
        let session_a = session_with_cookie("a", "sid", "same");
        let session_b = session_with_cookie("b", "sid", "same");
        let state = compose_playwright_state(&[session_a, session_b]).unwrap();

        assert_eq!(state.cookies.len(), 1);
    }

    #[test]
    fn rejects_conflicting_cookies() {
        let session_a = session_with_cookie("a", "sid", "first");
        let session_b = session_with_cookie("b", "sid", "second");
        let error = compose_playwright_state(&[session_a, session_b]).unwrap_err();

        assert_eq!(error.code(), ErrorCode::SessionConflict);
    }

    #[test]
    fn deduplicates_identical_origins() {
        let mut session_a = Session::new("a");
        session_a
            .origins
            .push(origin("https://example.com", "token", "same"));
        let mut session_b = Session::new("b");
        session_b
            .origins
            .push(origin("https://example.com", "token", "same"));

        let state = compose_playwright_state(&[session_a, session_b]).unwrap();
        assert_eq!(state.origins.len(), 1);
    }

    #[test]
    fn rejects_conflicting_origins() {
        let mut session_a = Session::new("a");
        session_a
            .origins
            .push(origin("https://example.com", "token", "first"));
        let mut session_b = Session::new("b");
        session_b
            .origins
            .push(origin("https://example.com", "token", "second"));

        let error = compose_playwright_state(&[session_a, session_b]).unwrap_err();
        assert_eq!(error.code(), ErrorCode::SessionConflict);
    }

    #[test]
    fn temp_state_file_is_removed_on_drop() {
        let temp = tempfile::tempdir().unwrap();
        let state = compose_playwright_state(&[]).unwrap();
        let temp_state = TempStateFile::write(temp.path(), &state).unwrap();
        let path = temp_state.path().to_path_buf();

        assert!(path.exists());
        drop(temp_state);
        assert!(!path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn temp_state_file_uses_private_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let state = compose_playwright_state(&[]).unwrap();
        let temp_state = TempStateFile::write(temp.path(), &state).unwrap();
        let mode = fs::metadata(temp_state.path())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;

        assert_eq!(mode, 0o600);
    }

    fn session_with_cookie(name: &str, cookie_name: &str, value: &str) -> Session {
        let mut session = Session::new(name);
        session.cookies.push(SessionCookie {
            name: cookie_name.to_string(),
            value: value.to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: true,
            secure: true,
            same_site: Some("Lax".to_string()),
            source_session: Some(name.to_string()),
        });
        session
    }

    fn origin(origin: &str, name: &str, value: &str) -> SessionOrigin {
        SessionOrigin {
            origin: origin.to_string(),
            local_storage: vec![StorageEntry {
                name: name.to_string(),
                value: value.to_string(),
            }],
            source_session: None,
        }
    }
}
