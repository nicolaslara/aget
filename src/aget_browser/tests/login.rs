use std::fs;

use crate::session::login::PendingLogin;

use super::super::*;

#[test]
fn cancels_pending_login_without_aget_facade_or_command_backend() {
    let temp = tempfile::tempdir().unwrap();
    let tmp_dir = temp.path().join("tmp");
    let profile = tmp_dir.join("owned-login/aget-docs");
    fs::create_dir_all(&profile).unwrap();
    fs::write(
        tmp_dir.join("login-docs.json"),
        serde_json::to_vec_pretty(&PendingLogin {
            name: "docs".to_string(),
            profile: profile.to_string_lossy().into_owned(),
            agent_session: "aget-login-docs".to_string(),
            url: "https://example.com/login".to_string(),
            allowed_domains: vec!["example.com".to_string()],
            injected_sessions: Vec::new(),
            browser_pid: None,
        })
        .unwrap(),
    )
    .unwrap();

    let cancelled = AgetBrowser::default()
        .cancel_login_session(LoginCancelOptions {
            name: "docs".to_string(),
            tmp_dir: tmp_dir.clone(),
        })
        .unwrap();

    assert_eq!(cancelled.pending.name, "docs");
    assert!(!profile.exists());
    assert!(!tmp_dir.join("login-docs.json").exists());
}
