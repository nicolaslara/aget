use aget::{Session, SessionCookie};

#[derive(serde::Serialize)]
pub(super) struct SessionView<'a> {
    ok: bool,
    version: u32,
    name: &'a str,
    sensitive: bool,
    allowed_cookie_domains: &'a [String],
    allowed_storage_origins: &'a [String],
    pub(super) cookies: Vec<CookieView<'a>>,
    pub(super) origins: Vec<OriginView<'a>>,
}

#[derive(serde::Serialize)]
pub(super) struct CookieView<'a> {
    pub(super) name: &'a str,
    pub(super) value: String,
    pub(super) domain: &'a str,
    path: &'a str,
    secure: bool,
    http_only: bool,
    pub(super) source_session: &'a Option<String>,
}

#[derive(serde::Serialize)]
pub(super) struct OriginView<'a> {
    pub(super) origin: &'a str,
    pub(super) local_storage: Vec<StorageEntryView<'a>>,
    pub(super) session_storage: Vec<StorageEntryView<'a>>,
    pub(super) source_session: &'a Option<String>,
}

#[derive(serde::Serialize)]
pub(super) struct StorageEntryView<'a> {
    pub(super) name: &'a str,
    pub(super) value: String,
}

pub(super) fn session_view(session: &Session, show_secrets: bool) -> SessionView<'_> {
    SessionView {
        ok: true,
        version: session.version,
        name: &session.name,
        sensitive: session.sensitive,
        allowed_cookie_domains: &session.allowed_cookie_domains,
        allowed_storage_origins: &session.allowed_storage_origins,
        cookies: session
            .cookies
            .iter()
            .map(|cookie| cookie_view(cookie, show_secrets))
            .collect(),
        origins: session
            .origins
            .iter()
            .map(|origin| origin_view(origin, show_secrets))
            .collect(),
    }
}

fn cookie_view(cookie: &SessionCookie, show_secrets: bool) -> CookieView<'_> {
    CookieView {
        name: &cookie.name,
        value: if show_secrets {
            cookie.value.clone()
        } else {
            "<redacted>".to_string()
        },
        domain: &cookie.domain,
        path: &cookie.path,
        secure: cookie.secure,
        http_only: cookie.http_only,
        source_session: &cookie.source_session,
    }
}

fn origin_view(origin: &aget::SessionOrigin, show_secrets: bool) -> OriginView<'_> {
    OriginView {
        origin: &origin.origin,
        local_storage: origin
            .local_storage
            .iter()
            .map(|entry| storage_entry_view(entry, show_secrets))
            .collect(),
        session_storage: origin
            .session_storage
            .iter()
            .map(|entry| storage_entry_view(entry, show_secrets))
            .collect(),
        source_session: &origin.source_session,
    }
}

fn storage_entry_view(entry: &aget::StorageEntry, show_secrets: bool) -> StorageEntryView<'_> {
    StorageEntryView {
        name: &entry.name,
        value: if show_secrets {
            entry.value.clone()
        } else {
            "<redacted>".to_string()
        },
    }
}

pub(super) fn source_suffix(source_session: &Option<String>) -> String {
    match source_session {
        Some(source) => format!(" source={source}"),
        None => String::new(),
    }
}
