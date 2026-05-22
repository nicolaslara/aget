use std::fs;
use std::path::PathBuf;

use aget::{
    complete_login_session, finish_login_session, start_login_session, LoginCompleteOptions,
    LoginFinishOptions, LoginStartOptions,
};
use serde_json::json;

use crate::support::session_cli::{agent_browser_tool, nyt_state};

#[test]
fn finish_login_session_leaves_pending_until_complete_login_session_runs() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let tmp_dir = aget_home.join("tmp");
    let log_path = temp.path().join("agent-browser.log");
    let fake_agent_browser = agent_browser_tool(
        temp.path(),
        &log_path,
        json!({"state": nyt_state("nyt_session", "token")}),
    );
    unsafe {
        std::env::set_var("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser);
        std::env::set_var("AGET_FAKE_AGENT_BROWSER_LOG", &log_path);
    }

    let start_result = start_login_session(LoginStartOptions {
        name: "hi".to_string(),
        profile: None,
        url: "https://www.nytimes.com/article".to_string(),
        tmp_dir: tmp_dir.clone(),
        injected_sessions: Vec::new(),
    })
    .unwrap();
    assert!(tmp_dir.join("login-hi.json").exists());
    assert_eq!(
        PathBuf::from(&start_result.pending.profile),
        tmp_dir.join("agent-browser/aget-hi")
    );
    let profile_dir = PathBuf::from(&start_result.pending.profile);
    fs::create_dir_all(&profile_dir).unwrap();

    let finish_result = finish_login_session(LoginFinishOptions {
        name: "hi".to_string(),
        tmp_dir: tmp_dir.clone(),
    })
    .unwrap();
    assert_eq!(finish_result.pending, start_result.pending);
    assert_eq!(finish_result.session.cookies[0].expires, Some(1812619153));
    assert!(tmp_dir.join("login-hi.json").exists());

    complete_login_session(LoginCompleteOptions {
        pending: finish_result.pending,
        tmp_dir,
    })
    .unwrap();

    assert!(!aget_home.join("tmp/login-hi.json").exists());
    assert!(!profile_dir.exists());
}
