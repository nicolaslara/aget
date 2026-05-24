use std::path::Path;

use aget::extraction::{ExtractorBackend, ExtractorBackendResult, ExtractorRequest};
use aget::{
    Aget, AgetError, ErrorCode, Session, SessionCookie, SessionOrigin, SessionStore, StorageEntry,
};

use super::mock_site::{MockResponse, MockSite};

pub(crate) fn aget(home: &Path) -> Aget {
    Aget::new(home)
}

pub(crate) fn success_data(output: &[u8], command: &str) -> serde_json::Value {
    let json: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], command);
    json["data"].clone()
}

pub(crate) fn storage_rendered_site() -> MockSite {
    MockSite::builder()
        .route(
            "/storage-rendered",
            MockResponse::html(
                r##"
<html>
  <body>
    <main><h1>Storage App Shell</h1><div id="storage-result"></div></main>
    <script>
      fetch("/storage-api", { headers: { "X-Local-Token": localStorage.getItem("local_token") }})
        .then((response) => response.text())
        .then((html) => { document.querySelector("#storage-result").innerHTML = html })
    </script>
  </body>
</html>
"##,
            ),
        )
        .start()
}

pub(crate) fn save_cookie_session(
    home: &Path,
    name: &str,
    domain: &str,
    cookie_name: &str,
    value: &str,
) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_cookie_domains.push(domain.to_string());
    session.cookies.push(SessionCookie {
        name: cookie_name.to_string(),
        value: value.to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}

pub(crate) fn save_storage_session(home: &Path, name: &str, origin: &str, key: &str, value: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_storage_origins.push(origin.to_string());
    session.origins.push(SessionOrigin {
        origin: origin.to_string(),
        local_storage: vec![StorageEntry {
            name: key.to_string(),
            value: value.to_string(),
        }],
        session_storage: Vec::new(),
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}

pub(crate) fn save_mixed_scope_session(home: &Path, name: &str, domain: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_cookie_domains.push(domain.to_string());
    session.cookies.push(SessionCookie {
        name: "app_session".to_string(),
        value: "valid-app".to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    session.cookies.push(SessionCookie {
        name: "other_session".to_string(),
        value: "other-secret".to_string(),
        domain: "unrelated.example".to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}

#[derive(Clone)]
pub(crate) struct FailingExtractor;

impl ExtractorBackend for FailingExtractor {
    fn name(&self) -> &'static str {
        "failing-extractor"
    }

    fn extract(&self, _request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        Err(AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: "primary extractor failed".to_string(),
        })
    }
}
