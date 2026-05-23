use std::collections::BTreeMap;

use crate::error::{AgetError, ErrorCode};
use crate::session::{Session, SessionCookie, SessionSource};

use super::command::run_cmux_cookies_get;
use super::domains::{domain_allowed, normalize_domain};
use super::types::CmuxImportOptions;

pub fn import_cmux_session(options: CmuxImportOptions) -> Result<Session, AgetError> {
    let mut cookies_by_key = BTreeMap::new();

    for domain in &options.domains {
        let response = run_cmux_cookies_get(&options.tmp_dir, &options.surface, domain)?;
        for cookie in response.cookies {
            if !domain_allowed(&cookie.domain, &options.domains) {
                continue;
            }

            let session_cookie = SessionCookie {
                name: cookie.name,
                value: cookie.value,
                domain: cookie.domain,
                path: cookie.path,
                expires: if cookie.session_only {
                    None
                } else {
                    cookie.expires
                },
                http_only: true,
                secure: cookie.secure,
                same_site: None,
                source_session: Some(options.name.clone()),
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
                            "cmux returned conflicting duplicate cookie '{}' for domain '{}' and path '{}'",
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
    }

    let mut session = Session::new(options.name);
    session.source = SessionSource::Cmux {
        surface: options.surface,
    };
    session.sensitive = true;
    session.allowed_cookie_domains = options.domains;
    session.cookies = cookies_by_key.into_values().collect();
    Ok(session)
}
