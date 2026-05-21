use crate::error::{AgetError, ErrorCode};
use crate::session::agent_browser::origin_host;
use crate::session::Session;

pub(super) fn enforce_replay_scope(url: &str, sessions: &[Session]) -> Result<(), AgetError> {
    if sessions.is_empty() {
        return Ok(());
    }

    let target_host = origin_host(url).ok_or_else(|| AgetError::Stable {
        code: ErrorCode::UsageError,
        message: format!("invalid request URL '{url}'"),
    })?;

    for session in sessions {
        if let Some(scope_error) = session_replay_scope_error(session, &target_host) {
            return Err(AgetError::Stable {
                code: ErrorCode::PrivacyPolicyBlocked,
                message: scope_error,
            });
        }
    }

    Ok(())
}

fn session_replay_scope_error(session: &Session, target_host: &str) -> Option<String> {
    let mut has_matching_scope = session
        .allowed_cookie_domains
        .iter()
        .any(|domain| domain_matches_host(target_host, domain));

    for cookie in &session.cookies {
        if domain_matches_host(target_host, &cookie.domain) {
            has_matching_scope = true;
        } else {
            return Some(format!(
                "session '{}' contains cookie state for '{}' outside request host '{}'",
                session.name,
                normalize_domain(&cookie.domain),
                target_host
            ));
        }
    }

    for origin in &session.allowed_storage_origins {
        if let Some(host) = origin_host(origin) {
            has_matching_scope |= domain_matches_host(target_host, &host);
        }
    }

    for origin in &session.origins {
        let host = origin_host(&origin.origin).unwrap_or_else(|| origin.origin.clone());
        if domain_matches_host(target_host, &host) {
            has_matching_scope = true;
        } else {
            return Some(format!(
                "session '{}' contains storage state for '{}' outside request host '{}'",
                session.name, origin.origin, target_host
            ));
        }
    }

    if has_matching_scope {
        None
    } else {
        Some(format!(
            "session '{}' is not scoped for request host '{}'",
            session.name, target_host
        ))
    }
}

pub(super) fn domain_matches_host(host: &str, allowed_domain: &str) -> bool {
    let host = normalize_domain(host);
    let allowed = normalize_domain(allowed_domain);
    if host.is_empty() || allowed.is_empty() {
        return false;
    }

    host == allowed || host.ends_with(&format!(".{allowed}"))
}

fn normalize_domain(domain: &str) -> String {
    domain
        .trim()
        .trim_start_matches('.')
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_scope_allows_subdomains_and_rejects_unrelated_hosts() {
        let mut session = Session::new("docs");
        session
            .allowed_cookie_domains
            .push("example.com".to_string());

        assert!(session_replay_scope_error(&session, "docs.example.com").is_none());
        assert!(session_replay_scope_error(&session, "example.com.evil").is_some());
    }

    #[test]
    fn replay_scope_rejects_mixed_domain_session_state() {
        let mut session = Session::new("mixed");
        session.cookies.push(crate::session::SessionCookie {
            name: "app".to_string(),
            value: "app-secret".to_string(),
            domain: "app.example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: true,
            secure: true,
            same_site: None,
            source_session: None,
        });
        session.cookies.push(crate::session::SessionCookie {
            name: "provider".to_string(),
            value: "provider-secret".to_string(),
            domain: "provider.example.com".to_string(),
            path: "/".to_string(),
            expires: None,
            http_only: true,
            secure: true,
            same_site: None,
            source_session: None,
        });

        let error = session_replay_scope_error(&session, "app.example.com").unwrap();

        assert!(error.contains("provider.example.com"));
    }
}
