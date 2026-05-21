use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{AgetError, ErrorCode};
use crate::session::{Session, SessionCookie, SessionOrigin, SessionSource, StorageEntry};

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
    let mut origins_by_name = BTreeMap::<String, OriginStorage>::new();

    for session in sessions {
        for cookie in &session.cookies {
            let playwright_cookie = normalize_playwright_cookie(PlaywrightCookie {
                name: cookie.name.clone(),
                value: cookie.value.clone(),
                domain: cookie.domain.clone(),
                path: cookie.path.clone(),
                expires: cookie.expires,
                http_only: cookie.http_only,
                secure: cookie.secure,
                same_site: cookie.same_site.clone(),
            });
            let key = CookieKey::from(&playwright_cookie);

            match cookies_by_key.get(&key) {
                Some(existing) if !same_playwright_cookie(existing, &playwright_cookie) => {
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
            let storage = origins_by_name.entry(origin.origin.clone()).or_default();
            merge_storage_entries(
                &origin.origin,
                "localStorage",
                &mut storage.local_storage,
                &origin.local_storage,
            )?;
            merge_storage_entries(
                &origin.origin,
                "sessionStorage",
                &mut storage.session_storage,
                &origin.session_storage,
            )?;
        }
    }

    Ok(PlaywrightState {
        cookies: cookies_by_key.into_values().collect(),
        origins: origins_by_name
            .into_iter()
            .map(|(origin, entries)| PlaywrightOrigin {
                origin,
                local_storage: entries.local_storage.into_values().collect(),
                session_storage: entries.session_storage.into_values().collect(),
            })
            .collect(),
    })
}

pub fn compose_session(name: &str, sessions: &[Session]) -> Result<Session, AgetError> {
    let mut cookies_by_key = BTreeMap::<CookieKey, SessionCookie>::new();
    let mut origins_by_name = BTreeMap::<String, ComposedOrigin>::new();
    let mut allowed_cookie_domains = BTreeSet::<String>::new();
    let mut allowed_storage_origins = BTreeSet::<String>::new();
    let source_names = sessions
        .iter()
        .map(|session| session.name.clone())
        .collect::<Vec<_>>();

    for session in sessions {
        allowed_cookie_domains.extend(session.allowed_cookie_domains.iter().cloned());
        allowed_storage_origins.extend(session.allowed_storage_origins.iter().cloned());

        for cookie in &session.cookies {
            let mut composed_cookie = normalize_session_cookie(cookie.clone());
            if composed_cookie.source_session.is_none() {
                composed_cookie.source_session = Some(session.name.clone());
            }
            let key = CookieKey::from(&composed_cookie);

            match cookies_by_key.get(&key) {
                Some(existing) if !same_cookie_for_composition(existing, &composed_cookie) => {
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
                    cookies_by_key.insert(key, composed_cookie);
                }
            }
        }

        for origin in &session.origins {
            let source = origin
                .source_session
                .clone()
                .unwrap_or_else(|| session.name.clone());
            let composed_origin = origins_by_name
                .entry(origin.origin.clone())
                .or_insert_with(|| ComposedOrigin::new(&origin.origin));
            composed_origin.sources.insert(source);
            merge_storage_entries(
                &origin.origin,
                "localStorage",
                &mut composed_origin.local_storage,
                &origin.local_storage,
            )?;
            merge_storage_entries(
                &origin.origin,
                "sessionStorage",
                &mut composed_origin.session_storage,
                &origin.session_storage,
            )?;
        }
    }

    let mut composed = Session::new(name);
    composed.source = SessionSource::Composed {
        sessions: source_names,
    };
    composed.sensitive = sessions.iter().any(|session| session.sensitive);
    composed.allowed_cookie_domains = allowed_cookie_domains.into_iter().collect();
    composed.allowed_storage_origins = allowed_storage_origins.into_iter().collect();
    composed.cookies = cookies_by_key.into_values().collect();
    composed.origins = origins_by_name
        .into_values()
        .map(ComposedOrigin::into_session_origin)
        .collect();
    Ok(composed)
}

#[derive(Debug)]
struct ComposedOrigin {
    origin: String,
    local_storage: BTreeMap<String, StorageEntry>,
    session_storage: BTreeMap<String, StorageEntry>,
    sources: BTreeSet<String>,
}

impl ComposedOrigin {
    fn new(origin: &str) -> Self {
        Self {
            origin: origin.to_string(),
            local_storage: BTreeMap::new(),
            session_storage: BTreeMap::new(),
            sources: BTreeSet::new(),
        }
    }

    fn into_session_origin(self) -> SessionOrigin {
        let source_session = if self.sources.len() == 1 {
            self.sources.into_iter().next()
        } else {
            None
        };
        SessionOrigin {
            origin: self.origin,
            local_storage: self.local_storage.into_values().collect(),
            session_storage: self.session_storage.into_values().collect(),
            source_session,
        }
    }
}

#[derive(Debug, Default)]
struct OriginStorage {
    local_storage: BTreeMap<String, StorageEntry>,
    session_storage: BTreeMap<String, StorageEntry>,
}

fn merge_storage_entries(
    origin: &str,
    storage_kind: &str,
    entries_by_name: &mut BTreeMap<String, StorageEntry>,
    entries: &[StorageEntry],
) -> Result<(), AgetError> {
    for entry in entries {
        match entries_by_name.get(&entry.name) {
            Some(existing) if existing.value != entry.value => {
                return Err(AgetError::Stable {
                    code: ErrorCode::SessionConflict,
                    message: format!(
                        "conflicting {storage_kind} key '{}' for origin '{}'",
                        entry.name, origin
                    ),
                });
            }
            Some(_) => {}
            None => {
                entries_by_name.insert(entry.name.clone(), entry.clone());
            }
        }
    }
    Ok(())
}

fn same_cookie_for_composition(left: &SessionCookie, right: &SessionCookie) -> bool {
    normalize_cookie_name(&left.name) == normalize_cookie_name(&right.name)
        && left.value == right.value
        && normalize_cookie_domain(&left.domain) == normalize_cookie_domain(&right.domain)
        && normalize_cookie_path(&left.path) == normalize_cookie_path(&right.path)
        && left.expires == right.expires
        && left.http_only == right.http_only
        && left.secure == right.secure
        && left.same_site == right.same_site
}

fn same_playwright_cookie(left: &PlaywrightCookie, right: &PlaywrightCookie) -> bool {
    normalize_cookie_name(&left.name) == normalize_cookie_name(&right.name)
        && left.value == right.value
        && normalize_cookie_domain(&left.domain) == normalize_cookie_domain(&right.domain)
        && normalize_cookie_path(&left.path) == normalize_cookie_path(&right.path)
        && left.expires == right.expires
        && left.http_only == right.http_only
        && left.secure == right.secure
        && left.same_site == right.same_site
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
            name: normalize_cookie_name(&cookie.name),
            domain: normalize_cookie_domain(&cookie.domain),
            path: normalize_cookie_path(&cookie.path),
        }
    }
}

impl From<&SessionCookie> for CookieKey {
    fn from(cookie: &SessionCookie) -> Self {
        Self {
            name: normalize_cookie_name(&cookie.name),
            domain: normalize_cookie_domain(&cookie.domain),
            path: normalize_cookie_path(&cookie.path),
        }
    }
}

fn normalize_cookie_name(name: &str) -> String {
    name.trim().to_string()
}

fn normalize_cookie_domain(domain: &str) -> String {
    domain
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

fn normalize_cookie_path(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() {
        "/".to_string()
    } else {
        path.to_string()
    }
}

fn normalize_playwright_cookie(mut cookie: PlaywrightCookie) -> PlaywrightCookie {
    cookie.name = normalize_cookie_name(&cookie.name);
    cookie.domain = normalize_cookie_domain(&cookie.domain);
    cookie.path = normalize_cookie_path(&cookie.path);
    cookie
}

fn normalize_session_cookie(mut cookie: SessionCookie) -> SessionCookie {
    cookie.name = normalize_cookie_name(&cookie.name);
    cookie.domain = normalize_cookie_domain(&cookie.domain);
    cookie.path = normalize_cookie_path(&cookie.path);
    cookie
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
        assert_eq!(state.cookies[0].name, "sid");
        assert_eq!(state.cookies[0].domain, "example.com");
        assert_eq!(state.cookies[0].path, "/");
    }

    #[test]
    fn deduplicates_cookies_with_normalized_identity() {
        let mut session_a = session_with_cookie("a", " sid ", "same");
        session_a.cookies[0].domain = ".Example.COM".to_string();
        session_a.cookies[0].path = String::new();
        let mut session_b = session_with_cookie("b", "sid", "same");
        session_b.cookies[0].domain = "example.com".to_string();
        session_b.cookies[0].path = "/".to_string();

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
    fn merges_disjoint_local_storage_keys_for_same_origin() {
        let mut session_a = Session::new("a");
        session_a
            .origins
            .push(origin("https://example.com", "token", "first"));
        let mut session_b = Session::new("b");
        session_b
            .origins
            .push(origin("https://example.com", "theme", "dark"));

        let state = compose_playwright_state(&[session_a, session_b]).unwrap();

        assert_eq!(state.origins.len(), 1);
        assert_eq!(state.origins[0].origin, "https://example.com");
        assert_eq!(state.origins[0].local_storage.len(), 2);
        assert!(state.origins[0]
            .local_storage
            .iter()
            .any(|entry| entry.name == "token" && entry.value == "first"));
        assert!(state.origins[0]
            .local_storage
            .iter()
            .any(|entry| entry.name == "theme" && entry.value == "dark"));
    }

    #[test]
    fn composes_session_storage_into_playwright_state() {
        let mut session = Session::new("a");
        let mut origin = origin("https://example.com", "token", "first");
        origin.session_storage.push(StorageEntry {
            name: "session-token".to_string(),
            value: "session-secret".to_string(),
        });
        session.origins.push(origin);

        let state = compose_playwright_state(&[session]).unwrap();

        assert_eq!(state.origins.len(), 1);
        assert_eq!(state.origins[0].session_storage.len(), 1);
        assert_eq!(state.origins[0].session_storage[0].name, "session-token");
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

    #[test]
    fn composed_session_preserves_and_fills_provenance_without_mutating_sources() {
        let mut provider = session_with_cookie("provider", "oauth", "provider-secret");
        provider.cookies[0].source_session = Some("browser-import".to_string());
        provider.allowed_cookie_domains = vec!["accounts.example.com".to_string()];
        provider.origins.push(origin(
            "https://accounts.example.com",
            "token",
            "provider-storage",
        ));
        let app = session_with_cookie("app", "appsid", "app-secret");
        let original_provider = provider.clone();
        let original_app = app.clone();

        let composed = compose_session("combined", &[provider.clone(), app.clone()]).unwrap();

        assert_eq!(
            composed.source,
            SessionSource::Composed {
                sessions: vec!["provider".to_string(), "app".to_string()]
            }
        );
        assert!(composed.cookies.iter().any(|cookie| {
            cookie.name == "oauth" && cookie.source_session.as_deref() == Some("browser-import")
        }));
        assert!(composed.cookies.iter().any(|cookie| {
            cookie.name == "appsid" && cookie.source_session.as_deref() == Some("app")
        }));
        assert_eq!(
            composed.origins[0].source_session.as_deref(),
            Some("provider")
        );
        assert_eq!(provider, original_provider);
        assert_eq!(app, original_app);
    }

    #[test]
    fn composed_session_merges_disjoint_local_storage_keys() {
        let mut session_a = Session::new("a");
        session_a
            .origins
            .push(origin("https://example.com", "token", "first"));
        let mut session_b = Session::new("b");
        session_b
            .origins
            .push(origin("https://example.com", "theme", "dark"));

        let composed = compose_session("combined", &[session_a, session_b]).unwrap();

        assert_eq!(composed.origins.len(), 1);
        assert_eq!(composed.origins[0].origin, "https://example.com");
        assert_eq!(composed.origins[0].local_storage.len(), 2);
        assert_eq!(composed.origins[0].source_session, None);
    }

    #[test]
    fn composed_session_merges_disjoint_session_storage_keys() {
        let mut session_a = Session::new("a");
        let mut origin_a = origin("https://example.com", "token", "first");
        origin_a.session_storage.push(StorageEntry {
            name: "session-token".to_string(),
            value: "first-session".to_string(),
        });
        session_a.origins.push(origin_a);
        let mut session_b = Session::new("b");
        let mut origin_b = origin("https://example.com", "theme", "dark");
        origin_b.session_storage.push(StorageEntry {
            name: "session-theme".to_string(),
            value: "dark-session".to_string(),
        });
        session_b.origins.push(origin_b);

        let composed = compose_session("combined", &[session_a, session_b]).unwrap();

        assert_eq!(composed.origins.len(), 1);
        assert_eq!(composed.origins[0].session_storage.len(), 2);
        assert_eq!(composed.origins[0].source_session, None);
    }

    #[test]
    fn composed_session_deduplicates_and_sorts_allowed_scopes() {
        let mut session_a = session_with_cookie("a", "sid", "same");
        session_a.allowed_cookie_domains =
            vec!["z.example.com".to_string(), "a.example.com".to_string()];
        session_a.allowed_storage_origins = vec!["https://z.example.com".to_string()];
        let mut session_b = session_with_cookie("b", "sid", "same");
        session_b.allowed_cookie_domains = vec!["a.example.com".to_string()];
        session_b.allowed_storage_origins = vec![
            "https://a.example.com".to_string(),
            "https://z.example.com".to_string(),
        ];

        let composed = compose_session("combined", &[session_a, session_b]).unwrap();

        assert_eq!(composed.cookies.len(), 1);
        assert_eq!(
            composed.allowed_cookie_domains,
            vec!["a.example.com".to_string(), "z.example.com".to_string()]
        );
        assert_eq!(
            composed.allowed_storage_origins,
            vec![
                "https://a.example.com".to_string(),
                "https://z.example.com".to_string()
            ]
        );
    }

    #[test]
    fn composed_session_deduplicates_cookies_with_normalized_identity() {
        let mut session_a = session_with_cookie("a", " sid ", "same");
        session_a.cookies[0].domain = ".Example.COM".to_string();
        session_a.cookies[0].path = String::new();
        let mut session_b = session_with_cookie("b", "sid", "same");
        session_b.cookies[0].domain = "example.com".to_string();
        session_b.cookies[0].path = "/".to_string();

        let composed = compose_session("combined", &[session_a, session_b]).unwrap();

        assert_eq!(composed.cookies.len(), 1);
        assert_eq!(composed.cookies[0].name, "sid");
        assert_eq!(composed.cookies[0].domain, "example.com");
        assert_eq!(composed.cookies[0].path, "/");
    }

    #[test]
    fn composed_session_rejects_conflicts_without_secret_values() {
        let session_a = session_with_cookie("a", "sid", "first-secret");
        let session_b = session_with_cookie("b", "sid", "second-secret");

        let error = compose_session("combined", &[session_a, session_b]).unwrap_err();
        let message = error.to_string();

        assert_eq!(error.code(), ErrorCode::SessionConflict);
        assert!(message.contains("conflicting cookie 'sid'"));
        assert!(!message.contains("first-secret"));
        assert!(!message.contains("second-secret"));
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
            session_storage: Vec::new(),
            source_session: None,
        }
    }
}
