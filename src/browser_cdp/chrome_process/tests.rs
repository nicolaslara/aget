use super::chrome_launch_command;
use crate::process::create_private_file;

#[test]
fn chrome_launch_args_include_agent_browser_stability_flags() {
    let temp = tempfile::tempdir().unwrap();
    let stderr = create_private_file(&temp.path().join("stderr.txt")).unwrap();
    let command = chrome_launch_command(
        temp.path().join("chrome").as_path(),
        &temp.path().join("profile"),
        None,
        false,
        true,
        None,
        stderr,
    );
    let args = command
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    for expected in [
        "--disable-hang-monitor",
        "--disable-prompt-on-repost",
        "--enable-features=NetworkService,NetworkServiceInProcess",
        "--metrics-recording-only",
    ] {
        assert!(
            args.iter().any(|arg| arg == expected),
            "missing Chrome launch arg {expected}; got {args:?}"
        );
    }
}

#[test]
fn chrome_launch_args_include_default_window_size_only_when_headless() {
    let temp = tempfile::tempdir().unwrap();
    let headless_stderr = create_private_file(&temp.path().join("headless-stderr.txt")).unwrap();
    let headless = chrome_launch_command(
        temp.path().join("chrome").as_path(),
        &temp.path().join("headless-profile"),
        None,
        false,
        true,
        None,
        headless_stderr,
    );
    let headless_args = headless
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(headless_args
        .iter()
        .any(|arg| arg == "--window-size=1280,720"));

    let headed_stderr = create_private_file(&temp.path().join("headed-stderr.txt")).unwrap();
    let headed = chrome_launch_command(
        temp.path().join("chrome").as_path(),
        &temp.path().join("headed-profile"),
        None,
        false,
        false,
        Some("https://example.com/login"),
        headed_stderr,
    );
    let headed_args = headed
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(
        !headed_args
            .iter()
            .any(|arg| arg.starts_with("--window-size=")),
        "headed Chrome launches should not receive a default window-size arg; got {headed_args:?}"
    );
    assert!(headed_args.iter().any(|arg| arg == "--new-window"));
    assert!(headed_args
        .iter()
        .any(|arg| arg == "https://example.com/login"));
}
