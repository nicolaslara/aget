use std::collections::BTreeMap;
use std::env;
use std::io;
use std::process::Command;

use serde::Deserialize;

use crate::error::{AgetError, ErrorCode};
use crate::session::{Session, SessionCookie, SessionSource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmuxImportOptions {
    pub surface: String,
    pub name: String,
    pub domains: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CmuxCookiesResponse {
    cookies: Vec<CmuxCookie>,
}

#[derive(Debug, Deserialize)]
struct CmuxCookie {
    name: String,
    value: String,
    domain: String,
    path: String,
    secure: bool,
    session_only: bool,
    expires: Option<i64>,
}

pub fn import_cmux_session(options: CmuxImportOptions) -> Result<Session, AgetError> {
    let mut cookies_by_key = BTreeMap::new();

    for domain in &options.domains {
        let response = run_cmux_cookies_get(&options.surface, domain)?;
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

fn run_cmux_cookies_get(surface: &str, domain: &str) -> Result<CmuxCookiesResponse, AgetError> {
    let command = env::var("AGET_CMUX_COMMAND").unwrap_or_else(|_| "cmux".to_string());
    let output = Command::new(&command)
        .args([
            "--json",
            "browser",
            "--surface",
            surface,
            "cookies",
            "get",
            "--domain",
            domain,
        ])
        .output()
        .map_err(|error| backend_unavailable(&command, error))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let code = if matches!(output.status.code(), Some(126 | 127)) {
            ErrorCode::BackendUnavailable
        } else {
            ErrorCode::ExtractionFailed
        };
        return Err(AgetError::Stable {
            code,
            message: if stderr.is_empty() {
                format!("cmux exited with {}", output.status)
            } else {
                stderr
            },
        });
    }

    serde_json::from_slice(&output.stdout).map_err(|error| AgetError::Stable {
        code: ErrorCode::ExtractionFailed,
        message: format!("cmux returned malformed JSON: {error}"),
    })
}

fn domain_allowed(cookie_domain: &str, allowed_domains: &[String]) -> bool {
    allowed_domains
        .iter()
        .any(|allowed| domain_matches_allowed(cookie_domain, allowed))
}

fn domain_matches_allowed(cookie_domain: &str, allowed_domain: &str) -> bool {
    let cookie = normalize_domain(cookie_domain);
    let allowed = normalize_domain(allowed_domain);
    if cookie.is_empty() || allowed.is_empty() {
        return false;
    }

    if cookie == allowed {
        return true;
    }

    if let Some(cookie_root) = cookie.strip_prefix('.') {
        return cookie_root == allowed;
    }

    is_domain_or_subdomain(&cookie, &allowed)
}

fn is_domain_or_subdomain(candidate: &str, root: &str) -> bool {
    candidate == root || candidate.ends_with(&format!(".{root}"))
}

fn normalize_domain(domain: &str) -> String {
    domain.trim().trim_end_matches('.').to_ascii_lowercase()
}

fn backend_unavailable(command: &str, error: io::Error) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::BackendUnavailable,
        message: format!("cmux backend is unavailable for command '{command}': {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_matching_rejects_substring_only_matches() {
        assert!(domain_matches_allowed("example.com", "example.com"));
        assert!(domain_matches_allowed(".example.com", "example.com"));
        assert!(!domain_matches_allowed(".example.com", "docs.example.com"));
        assert!(domain_matches_allowed(
            ".docs.example.com",
            "docs.example.com"
        ));
        assert!(domain_matches_allowed("docs.example.com", "example.com"));
        assert!(!domain_matches_allowed("example.com", "docs.example.com"));
        assert!(!domain_matches_allowed("evil-example.com", "example.com"));
        assert!(!domain_matches_allowed("example.com.evil", "example.com"));
    }
}
