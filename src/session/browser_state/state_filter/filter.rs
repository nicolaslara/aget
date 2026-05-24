use super::domains::{domain_allowed, normalize_domain, origin_host};
use super::model::{BrowserCookie, BrowserOrigin, BrowserState};
use crate::error::{AgetError, ErrorCode};
use crate::session::{PlaywrightState, Session, SessionCookie, SessionOrigin, SessionSource};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct BrowserSessionFilter {
    pub(crate) name: String,
    pub(crate) source: SessionSource,
    pub(crate) allowed_domains: Vec<String>,
    pub(crate) source_session: String,
}

pub(crate) fn filter_browser_state(
    state: BrowserState,
    filter: BrowserSessionFilter,
) -> Result<Session, AgetError> {
    let mut cookies_by_key = BTreeMap::new();
    for cookie in state.cookies {
        if !domain_allowed(&cookie.domain, &filter.allowed_domains) {
            continue;
        }

        let session_cookie = SessionCookie {
            name: cookie.name,
            value: cookie.value,
            domain: cookie.domain,
            path: cookie.path,
            expires: cookie.expires,
            http_only: cookie.http_only,
            secure: cookie.secure,
            same_site: cookie.same_site,
            source_session: Some(filter.source_session.clone()),
        };
        let key = (
            session_cookie.name.clone(),
            normalize_domain(&session_cookie.domain),
            session_cookie.path.clone(),
        );
        match cookies_by_key.get(&key) {
            Some(existing) if existing != &session_cookie => {
                return Err(AgetError::Stable {
                    code: ErrorCode::SessionConflict,
                    message: format!(
                        "browser state returned conflicting duplicate cookie '{}' for domain '{}' and path '{}'",
                        key.0, key.1, key.2
                    ),
                });
            }
            Some(_) => {}
            None => {
                cookies_by_key.insert(key, session_cookie);
            }
        }
    }

    let mut origins_by_name = BTreeMap::new();
    for origin in state.origins {
        let Some(host) = origin_host(&origin.origin) else {
            continue;
        };
        if !domain_allowed(&host, &filter.allowed_domains) {
            continue;
        }

        let session_origin = SessionOrigin {
            origin: origin.origin,
            local_storage: origin.local_storage,
            session_storage: origin.session_storage,
            source_session: Some(filter.source_session.clone()),
        };
        match origins_by_name.get(&session_origin.origin) {
            Some(existing) if existing != &session_origin => {
                return Err(AgetError::Stable {
                    code: ErrorCode::SessionConflict,
                    message: format!(
                        "browser state returned conflicting duplicate storage origin '{}'",
                        session_origin.origin
                    ),
                });
            }
            Some(_) => {}
            None => {
                origins_by_name.insert(session_origin.origin.clone(), session_origin);
            }
        }
    }

    let mut session = Session::new(filter.name);
    session.source = filter.source;
    session.sensitive = true;
    session.allowed_cookie_domains = filter.allowed_domains;
    session.cookies = cookies_by_key.into_values().collect();
    session.origins = origins_by_name.into_values().collect();
    session.allowed_storage_origins = session
        .origins
        .iter()
        .map(|origin| origin.origin.clone())
        .collect();
    Ok(session)
}

pub(crate) fn filter_playwright_state(
    state: PlaywrightState,
    filter: BrowserSessionFilter,
) -> Result<Session, AgetError> {
    filter_browser_state(
        BrowserState {
            cookies: state
                .cookies
                .into_iter()
                .map(|cookie| BrowserCookie {
                    name: cookie.name,
                    value: cookie.value,
                    domain: cookie.domain,
                    path: cookie.path,
                    expires: cookie.expires,
                    http_only: cookie.http_only,
                    secure: cookie.secure,
                    same_site: cookie.same_site,
                })
                .collect(),
            origins: state
                .origins
                .into_iter()
                .map(|origin| BrowserOrigin {
                    origin: origin.origin,
                    local_storage: origin.local_storage,
                    session_storage: origin.session_storage,
                })
                .collect(),
        },
        filter,
    )
}
