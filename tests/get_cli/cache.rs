use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;

use crate::support::get_cli::{save_cookie_session, success_data};
use crate::support::mock_site::{MockResponse, MockSite};

#[test]
fn get_reuses_public_http_cache_without_changing_content() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/cache",
            MockResponse::html("<main><h1>Cache</h1><p>Exact content.</p></main>"),
        )
        .start();
    let url = site.url("/cache");

    let first = get_json(&aget_home, &[&url]);
    assert_eq!(first["cache"]["status"], "miss");
    assert_eq!(first["cache"]["policy"], "auto");
    assert_eq!(first["cache"]["eligible"], true);
    assert_eq!(first["content"], "# Cache\n\nExact content.");
    assert!(first["cache"]["key"].as_str().is_some());
    assert!(first["usage"]["fetched_bytes"].as_u64().unwrap() > 0);
    assert!(first["usage"]["content_bytes"].as_u64().unwrap() > 0);
    assert!(first["usage"]["estimated_tokens"].as_u64().unwrap() > 0);
    assert!(first["usage"]["estimated_tokens_saved"].as_u64().is_some());

    let second = get_json(&aget_home, &[&url]);
    assert_eq!(second["cache"]["status"], "hit");
    assert_eq!(second["cache"]["eligible"], true);
    assert_eq!(second["cache"]["key"], first["cache"]["key"]);
    assert_eq!(second["content"], first["content"]);
    assert_eq!(site.requests().len(), 1);

    let metadata_path = PathBuf::from(second["artifacts"]["metadata"].as_str().unwrap());
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["cache"], second["cache"]);
    assert_eq!(metadata["usage"], second["usage"]);
}

#[test]
fn get_fresh_and_zero_ttl_refresh_public_http_cache() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/fresh",
            MockResponse::html("<main><h1>Fresh</h1><p>Refresh me.</p></main>"),
        )
        .start();
    let url = site.url("/fresh");

    assert_eq!(get_json(&aget_home, &[&url])["cache"]["status"], "miss");

    let fresh = get_json(&aget_home, &[&url, "--fresh"]);
    assert_eq!(fresh["cache"]["status"], "refresh");
    assert_eq!(fresh["cache"]["policy"], "refresh");
    assert_eq!(site.requests().len(), 2);

    let stale = get_json(&aget_home, &[&url, "--cache-ttl", "0"]);
    assert_eq!(stale["cache"]["status"], "stale");
    assert_eq!(stale["cache"]["ttl_seconds"], 0);
    assert_eq!(site.requests().len(), 3);
}

#[test]
fn get_does_not_cache_session_backed_content() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/private",
            MockResponse::html("<main><h1>Private</h1><p>Session content.</p></main>"),
        )
        .start();
    save_cookie_session(&aget_home, "docs", &site.host(), "sid", "secret");
    let url = site.url("/private");

    let first = get_json(&aget_home, &[&url, "--session", "docs"]);
    assert_eq!(first["cache"]["status"], "ineligible");
    assert_eq!(first["cache"]["eligible"], false);
    assert_eq!(
        first["cache"]["reason"],
        "session-backed requests are not reusable across cache scopes"
    );

    let second = get_json(&aget_home, &[&url, "--session", "docs"]);
    assert_eq!(second["cache"]["status"], "ineligible");
    assert_eq!(site.requests().len(), 2);
    assert!(fs::read_dir(aget_home.join("cache"))
        .unwrap()
        .next()
        .is_none());
}

fn get_json(aget_home: &std::path::Path, args: &[&str]) -> serde_json::Value {
    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", aget_home)
        .args(["--envelope", "json", "get"])
        .args(args)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    success_data(&output, "get")
}
