use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::OnceLock;

use aget::extraction::{ExtractorBackend, ExtractorBackendResult, ExtractorRequest};
use aget::{
    Aget, AgetError, ErrorCode, Session, SessionCookie, SessionOrigin, SessionStore, StorageEntry,
};

use super::mock_site::{MockResponse, MockSite};

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub(crate) fn mock_backend_command() -> String {
    shell_quote(&mock_tool_path("aget-mock-backend").to_string_lossy())
}

pub(crate) fn mock_agent_browser_command() -> PathBuf {
    mock_tool_path("aget-mock-agent-browser")
}

fn mock_tool_path(name: &str) -> PathBuf {
    let tools = MOCK_TOOLS.get_or_init(build_mock_tools);
    let binary = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    tools.target_dir.join("debug").join(binary)
}

struct MockTools {
    target_dir: PathBuf,
}

static MOCK_TOOLS: OnceLock<MockTools> = OnceLock::new();

fn build_mock_tools() -> MockTools {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = root.join("tests/fixtures/mock-tools/Cargo.toml");
    let target_dir = root.join("target/aget-mock-tools");
    let status = StdCommand::new("cargo")
        .args([
            "build",
            "--quiet",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--target-dir",
            target_dir.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "failed to build mocked e2e helper tools");
    MockTools { target_dir }
}

pub(crate) fn aget(home: &Path, backend: &str) -> Aget {
    Aget::new(home).with_backend_command(backend.to_string())
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
