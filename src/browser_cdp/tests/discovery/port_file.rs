use std::fs;
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::browser_cdp::discovery::{
    connect_existing_profile_browser, read_devtools_active_port, wait_for_devtools_active_port,
};
use crate::browser_cdp::process::terminate_child;
use crate::error::ErrorCode;
use crate::process::{create_private_file, TempOutputFile};

#[test]
fn wait_for_devtools_active_port_reports_early_exit_code() {
    let temp = tempfile::tempdir().unwrap();
    let stderr_capture = TempOutputFile::new(temp.path(), "chrome-stderr").unwrap();
    let stderr = create_private_file(stderr_capture.path()).unwrap();
    let mut child = early_exit_command(7)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr))
        .spawn()
        .unwrap();

    let error = wait_for_devtools_active_port(
        &mut child,
        temp.path(),
        &stderr_capture,
        Duration::from_secs(2),
    )
    .unwrap_err();

    assert_eq!(error.code(), ErrorCode::BackendUnavailable);
    assert!(error
        .to_string()
        .contains("Chrome exited early (exit code: 7) without writing DevToolsActivePort"));
}

#[cfg(unix)]
fn early_exit_command(code: i32) -> Command {
    let mut command = Command::new("sh");
    command.arg("-c").arg(format!("exit {code}"));
    command
}

#[cfg(windows)]
fn early_exit_command(code: i32) -> Command {
    let mut command = Command::new("cmd");
    command.arg("/C").arg(format!("exit /B {code}"));
    command
}

#[cfg(unix)]
#[test]
fn wait_for_devtools_active_port_uses_stderr_fallback() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let stderr_capture = TempOutputFile::new(temp.path(), "chrome-stderr").unwrap();
    let stderr = create_private_file(stderr_capture.path()).unwrap();
    let fake_chrome = temp.path().join("fake-chrome");
    fs::write(
        &fake_chrome,
        "#!/bin/sh\n\
         echo 'DevTools listening on ws://127.0.0.1:9222/devtools/browser/fallback' >&2\n\
         sleep 5\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_chrome).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&fake_chrome, permissions).unwrap();
    let mut child = Command::new(&fake_chrome)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr))
        .spawn()
        .unwrap();

    let ws_url = wait_for_devtools_active_port(
        &mut child,
        temp.path(),
        &stderr_capture,
        Duration::from_secs(5),
    )
    .unwrap();
    terminate_child(&mut child);

    assert_eq!(ws_url, "ws://127.0.0.1:9222/devtools/browser/fallback");
}

#[test]
fn parses_devtools_active_port_file() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(
        temp.path().join("DevToolsActivePort"),
        "49152\n/devtools/browser/abc\n",
    )
    .unwrap();

    assert_eq!(
        read_devtools_active_port(temp.path()),
        Some((49152, "/devtools/browser/abc".to_string()))
    );
}

#[test]
fn existing_profile_attach_removes_stale_devtools_active_port() {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let temp = tempfile::tempdir().unwrap();
    let active_port = temp.path().join("DevToolsActivePort");
    fs::write(&active_port, format!("{port}\n/devtools/browser/stale\n")).unwrap();

    let client = connect_existing_profile_browser(temp.path(), Duration::from_millis(100))
        .expect("stale CDP attach should not be fatal");

    assert!(client.is_none());
    assert!(!active_port.exists());
}
