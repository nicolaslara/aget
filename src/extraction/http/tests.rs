use std::time::Duration;

use crate::session::PlaywrightState;

use super::owned_fetch;

fn empty_state() -> PlaywrightState {
    PlaywrightState {
        cookies: Vec::new(),
        origins: Vec::new(),
    }
}

#[test]
fn owned_fetch_accepts_raw_colon_html_without_url_parsing() {
    let response = owned_fetch(
        "raw:<html><body><main>Raw</main></body></html>",
        &empty_state(),
        Duration::from_secs(1),
        None,
    )
    .unwrap();

    assert_eq!(
        response.final_url,
        "raw:<html><body><main>Raw</main></body></html>"
    );
    assert_eq!(response.body, "<html><body><main>Raw</main></body></html>");
    assert!(!response.can_auto_render);
}

#[test]
fn owned_fetch_accepts_raw_slash_html_without_url_parsing() {
    let response = owned_fetch(
        "raw://<html><body><main>Raw slash</main></body></html>",
        &empty_state(),
        Duration::from_secs(1),
        None,
    )
    .unwrap();

    assert_eq!(
        response.body,
        "<html><body><main>Raw slash</main></body></html>"
    );
    assert!(!response.can_auto_render);
}

#[test]
fn owned_fetch_reads_explicit_file_url() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("page.html");
    std::fs::write(&path, "<html><body><main>File</main></body></html>").unwrap();
    let url = format!("file://{}", path.display());

    let response = owned_fetch(&url, &empty_state(), Duration::from_secs(1), None).unwrap();

    assert_eq!(response.final_url, url);
    assert_eq!(response.body, "<html><body><main>File</main></body></html>");
    assert!(!response.can_auto_render);
}
