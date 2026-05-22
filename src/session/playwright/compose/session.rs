use std::collections::{BTreeMap, BTreeSet};

use crate::error::{AgetError, ErrorCode};
use crate::session::{Session, SessionCookie, SessionSource};

use super::cookies::{normalize_session_cookie, same_cookie_for_composition, CookieKey};
use super::storage::{merge_storage_entries, ComposedOrigin};

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
            composed_origin.insert_source(source);
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
