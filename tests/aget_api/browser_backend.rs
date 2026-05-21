use std::fs;

use aget::{Aget, AgetBrowserBackend, ErrorCode};

use crate::support::pending_login;

#[test]
fn aget_browser_backend_does_not_save_failed_profile_path_import() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let missing_profile = temp.path().join("missing-profile");

    let error = Aget::new(&home)
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .import_chrome_session(
            missing_profile.to_string_lossy().to_string(),
            "chrome",
            vec!["example.com".to_string()],
        )
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::RequiresUserAction);
    assert!(error.to_string().contains("does not exist"));
    assert!(!home.join("sessions/chrome.json").exists());
}

#[test]
fn aget_browser_backend_cancels_pending_login_without_agent_browser() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("aget-home");
    let tmp_dir = home.join("tmp");
    let profile = tmp_dir.join("owned-login/aget-docs");
    fs::create_dir_all(&profile).unwrap();
    fs::create_dir_all(&tmp_dir).unwrap();
    fs::write(
        tmp_dir.join("login-docs.json"),
        serde_json::to_vec_pretty(&pending_login(
            "docs".to_string(),
            profile.to_string_lossy().into_owned(),
            "https://example.com/login".to_string(),
        ))
        .unwrap(),
    )
    .unwrap();

    let cancelled = Aget::new(&home)
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .cancel_login_session("docs")
        .unwrap();

    assert_eq!(cancelled.pending.name, "docs");
    assert!(!profile.exists());
    assert!(!tmp_dir.join("login-docs.json").exists());
}
