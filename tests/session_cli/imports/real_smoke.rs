use aget::session::SessionStore;
use aget::SessionSource;
use assert_cmd::Command;

use crate::support::session_cli::success_data;

#[test]
#[ignore = "requires local Chrome/Chromium plus explicit AGET_REAL_BROWSER_PROFILE and AGET_REAL_BROWSER_DOMAIN"]
fn real_session_import_browser_chrome_profile() {
    let (profile, domain) = match (
        std::env::var("AGET_REAL_BROWSER_PROFILE"),
        std::env::var("AGET_REAL_BROWSER_DOMAIN"),
    ) {
        (Ok(profile), Ok(domain)) if !profile.trim().is_empty() && !domain.trim().is_empty() => {
            (profile, domain)
        }
        _ => {
            eprintln!(
                "set AGET_REAL_BROWSER_PROFILE and AGET_REAL_BROWSER_DOMAIN to an approved logged-in Chrome profile and scoped domain"
            );
            return;
        }
    };
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "browser",
            "--browser",
            "chrome",
            "--browser-profile",
            &profile,
            "--name",
            "real-browser-import",
            "--allow-domain",
            &domain,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "session.import.browser");
    assert_eq!(json["source"], "browser_profile");
    assert_eq!(json["browser"], "chrome");
    assert_eq!(json["name"], "real-browser-import");
    assert!(
        json["cookie_count"].as_u64().unwrap_or(0) > 0
            || json["origin_count"].as_u64().unwrap_or(0) > 0,
        "real import saved no scoped browser auth state"
    );

    let store = SessionStore::new(&aget_home).unwrap();
    let session = store.load("real-browser-import").unwrap();
    assert_eq!(
        session.source,
        SessionSource::ChromeProfile {
            profile: profile.clone()
        }
    );
    assert!(session.sensitive);
    assert_eq!(session.allowed_cookie_domains, vec![domain]);
    assert!(
        !session.cookies.is_empty() || !session.origins.is_empty(),
        "real import persisted no scoped cookies or storage"
    );
}
