use crate::session::{Session, SessionCookie, SessionOrigin, StorageEntry};

pub(super) fn session_with_cookie(name: &str, cookie_name: &str, value: &str) -> Session {
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

pub(super) fn origin(origin: &str, name: &str, value: &str) -> SessionOrigin {
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
