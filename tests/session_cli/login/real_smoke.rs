use assert_cmd::Command;

use crate::support::session_cli::success_data;

#[test]
#[ignore = "requires local agent-browser, Crawl4AI setup, and manual authorized HelloInterview login"]
fn real_hellointerview_login_flow_fetches_paywalled_markdown() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let target = std::env::var("AGET_REAL_HELLOINTERVIEW_URL").unwrap_or_else(|_| {
        "https://www.hellointerview.com/learn/behavioral/course/select-choosing-responses-strategically".to_string()
    });

    let mut start = Command::cargo_bin("aget").unwrap();
    start
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "hellointerview",
            "--url",
            &target,
        ])
        .assert()
        .success();

    eprintln!(
        "Complete the HelloInterview/Google login in the opened browser, then press Enter here."
    );
    let mut confirmation = String::new();
    std::io::stdin().read_line(&mut confirmation).unwrap();

    let mut finish = Command::cargo_bin("aget").unwrap();
    finish
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "finish",
            "hellointerview",
        ])
        .assert()
        .success();

    let mut get = Command::cargo_bin("aget").unwrap();
    let output = get
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "get",
            &target,
            "--session",
            "hellointerview",
            "--content-format",
            "markdown",
            "--timeout",
            "90",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    let content = json["content"].as_str().unwrap();
    assert!(!content.contains("Purchase Premium to Keep Reading"));
    assert!(!content.contains("Premium users can view this video once signed in"));
    assert!(content.len() > 500);
}
