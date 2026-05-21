#[path = "aget_extractor/basic_formats.rs"]
mod basic_formats;
#[path = "aget_extractor/cleanup.rs"]
mod cleanup;
#[path = "aget_extractor/main_content.rs"]
mod main_content;
#[path = "aget_extractor/markdown.rs"]
mod markdown;
#[path = "aget_extractor/options_waits.rs"]
mod options_waits;
#[path = "aget_extractor/selectors.rs"]
mod selectors;

use crate::aget_extractor_site::aget_extractor_parity_site;
use crate::support::mock_site_cli::save_cookie_session;

#[test]
fn aget_extractor_backend_covers_static_http_parity_slice() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = aget_extractor_parity_site();
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    basic_formats::assert_public_session_and_formats(&aget_home, &site);
    markdown::assert_markdown_rendering(&aget_home, &site);
    cleanup::assert_cleanup_outputs(&aget_home, &site);
    main_content::assert_main_content_scoring(&aget_home, &site);
    selectors::assert_selector_and_target_options(&aget_home, &site);
    options_waits::assert_backend_options_redirects_and_waits(&aget_home, &site);
}
