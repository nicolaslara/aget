use aget::session::login::PendingLogin;
use aget::{Session, SessionCookie, SessionSource};

pub(crate) fn cookie_session(name: &str, domain: &str, value: &str) -> Session {
    let mut session = Session::new(name);
    session.source = SessionSource::Manual;
    session.allowed_cookie_domains = vec![domain.to_string()];
    session.cookies = vec![SessionCookie {
        name: "sid".to_string(),
        value: value.to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: true,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    }];
    session
}

pub(crate) fn pending_login(
    name: String,
    profile: String,
    url: String,
    injected_sessions: Vec<String>,
) -> PendingLogin {
    PendingLogin {
        agent_session: format!("aget-login-{name}"),
        allowed_domains: vec!["example.com".to_string()],
        browser_pid: None,
        injected_sessions,
        name,
        profile,
        url,
    }
}
