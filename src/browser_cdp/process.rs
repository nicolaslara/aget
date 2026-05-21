use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{AgetError, ErrorCode};

pub(super) fn ensure_login_browser_exited(
    profile_dir: &Path,
    pid: Option<u32>,
    timeout: Duration,
) -> Result<(), AgetError> {
    let Some(pid) = pid else {
        return Ok(());
    };

    wait_for_process_exit(pid, timeout);
    if !process_is_running(pid) {
        return Ok(());
    }

    if !process_command_mentions(pid, profile_dir) {
        return Ok(());
    }

    terminate_process_group_or_pid(pid);
    wait_for_process_exit(pid, Duration::from_secs(2));
    if process_is_running(pid) {
        return Err(AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: format!("owned login browser process {pid} did not exit after Browser.close"),
        });
    }
    Ok(())
}

fn wait_for_process_exit(pid: u32, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !process_is_running(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(unix)]
fn process_is_running(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(not(unix))]
fn process_is_running(_pid: u32) -> bool {
    #[cfg(windows)]
    {
        let filter = format!("PID eq {_pid}");
        return Command::new("tasklist")
            .args(["/FI", &filter, "/NH"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .is_some_and(|output| output.contains(&_pid.to_string()));
    }
    false
}

#[cfg(unix)]
fn process_command_mentions(pid: u32, needle: &Path) -> bool {
    Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .is_some_and(|command| command.contains(&needle.to_string_lossy().into_owned()))
}

#[cfg(not(unix))]
fn process_command_mentions(pid: u32, _needle: &Path) -> bool {
    #[cfg(windows)]
    {
        // The pid is captured from a Chrome process launched by this engine.
        // agent-browser also uses pid-based Windows cleanup for detached
        // processes, so do not make cleanup depend on fragile command-line
        // probing APIs that are not always available on Windows.
        let _ = _needle;
        return process_is_running(pid);
    }
    false
}

#[cfg(unix)]
fn terminate_process_group_or_pid(pid: u32) {
    let process_group = format!("-{pid}");
    let pid = pid.to_string();
    let _ = Command::new("kill")
        .args(["-TERM", &process_group])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("kill")
        .args(["-TERM", &pid])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    thread::sleep(Duration::from_millis(100));
    let _ = Command::new("kill")
        .args(["-KILL", &process_group])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = Command::new("kill")
        .args(["-KILL", &pid])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(not(unix))]
fn terminate_process_group_or_pid(pid: u32) {
    #[cfg(windows)]
    {
        let args = windows_taskkill_args(pid);
        let _ = Command::new("taskkill")
            .args(args.iter().map(String::as_str))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        thread::sleep(Duration::from_millis(500));
    }
}

pub(super) fn configure_chrome_process_group(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    #[cfg(not(unix))]
    {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(windows_detached_process_flags());
        }
        #[cfg(not(windows))]
        {
            let _ = command;
        }
    }
}

pub(super) fn terminate_child(child: &mut Child) {
    #[cfg(unix)]
    {
        let process_group = format!("-{}", child.id());
        let _ = Command::new("kill")
            .args(["-TERM", &process_group])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        thread::sleep(Duration::from_millis(50));
        let _ = Command::new("kill")
            .args(["-KILL", &process_group])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(not(unix))]
    {
        #[cfg(windows)]
        {
            let args = windows_taskkill_args(child.id());
            let _ = Command::new("taskkill")
                .args(args.iter().map(String::as_str))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        let _ = child.kill();
    }
    let _ = child.wait();
}

#[cfg(any(windows, test))]
fn windows_detached_process_flags() -> u32 {
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
    const DETACHED_PROCESS: u32 = 0x00000008;
    CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS
}

#[cfg(any(windows, test))]
fn windows_taskkill_args(pid: u32) -> [String; 3] {
    ["/PID".to_string(), pid.to_string(), "/F".to_string()]
}

#[cfg(test)]
mod tests {
    use super::{windows_detached_process_flags, windows_taskkill_args};

    #[test]
    fn windows_process_flags_match_agent_browser_detached_group_launch() {
        assert_eq!(windows_detached_process_flags(), 0x00000200 | 0x00000008);
    }

    #[test]
    fn windows_taskkill_args_match_agent_browser_pid_termination() {
        assert_eq!(windows_taskkill_args(42), ["/PID", "42", "/F"]);
    }
}
