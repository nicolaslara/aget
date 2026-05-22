use aget::{Aget, AgetBrowserBackend, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site_cli::{save_storage_session, storage_rendered_site};

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_extractor_backend_renders_local_storage_backed_session_with_chrome() {
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

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .get(site.url("/storage-rendered"))
        .session("storage")
        .content_format(OutputFormat::Text)
        .wait_for_selector("#storage-result h1")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Storage App Shell\nStorage Protected");
    assert!(site.received_header("/storage-api", "x-local-token", "storage-secret"));
}
