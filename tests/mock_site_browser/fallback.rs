use aget::{Aget, AgetBrowserBackend, OutputFormat};

use crate::support::mock_site::{MockResponse, MockSite};
use crate::support::mock_site_cli::{
    save_cookie_session, save_storage_session, storage_rendered_site, FailingExtractor,
};

#[test]
fn aget_browser_fallback_replays_cookie_backed_session_without_browser_state() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let fallback = Aget::new(&aget_home)
        .with_extractor_backend(FailingExtractor)
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .get(site.url("/protected"))
        .session("app")
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();

    assert_eq!(fallback.extractor, "aget-owned-browser-fallback");
    assert_eq!(fallback.content, "Protected Account\nPrivate account body.");
    assert_eq!(
        fallback.page_metadata["title"],
        "Protected Account Metadata"
    );
    assert_eq!(
        fallback.page_metadata["description"],
        "Protected account description"
    );
    assert_eq!(fallback.page_metadata["og:title"], "Protected Account OG");
    assert_eq!(
        fallback.warnings,
        vec!["aget-owned fallback used after primary extractor failed"]
    );
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_browser_fallback_renders_cookie_backed_scripted_page_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/client-rendered",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Client Shell</h1>
      <div id="client-result">Loading</div>
    </main>
    <script>
      setTimeout(() => {
        document.querySelector("#client-result").innerHTML = "<p>Client Rendered</p>"
      }, 25)
    </script>
  </body>
</html>
"##,
            ),
        )
        .start();
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let fallback = Aget::new(&aget_home)
        .with_extractor_backend(FailingExtractor)
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .get(site.url("/client-rendered"))
        .session("app")
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();

    assert_eq!(fallback.extractor, "aget-owned-browser-fallback");
    assert_eq!(fallback.content, "Client Shell\nClient Rendered");
    assert!(site.received_cookie("/client-rendered", "app_session", "valid-app"));
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_browser_fallback_renders_local_storage_backed_session_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = storage_rendered_site();
    save_storage_session(
        &aget_home,
        "storage",
        &site.origin(),
        "local_token",
        "storage-secret",
    );

    let fallback = Aget::new(&aget_home)
        .with_extractor_backend(FailingExtractor)
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .get(site.url("/storage-rendered"))
        .session("storage")
        .content_format(OutputFormat::Text)
        .wait_for_selector("#storage-result h1")
        .run()
        .unwrap();

    assert_eq!(fallback.extractor, "aget-owned-browser-fallback");
    assert_eq!(fallback.content, "Storage App Shell\nStorage Protected");
    assert!(site.received_header("/storage-api", "x-local-token", "storage-secret"));
}
