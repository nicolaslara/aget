use std::collections::{BTreeMap, BTreeSet};

use crate::error::{AgetError, ErrorCode};
use crate::session::{Session, SessionCookie, SessionOrigin, SessionSource, StorageEntry};

use super::{PlaywrightCookie, PlaywrightOrigin, PlaywrightState};

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
