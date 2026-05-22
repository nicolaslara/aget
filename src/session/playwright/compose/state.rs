use std::collections::BTreeMap;

use crate::error::{AgetError, ErrorCode};
use crate::session::Session;

use super::super::{PlaywrightCookie, PlaywrightOrigin, PlaywrightState};
use super::cookies::{normalize_playwright_cookie, same_playwright_cookie, CookieKey};
use super::storage::{merge_storage_entries, OriginStorage};

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
