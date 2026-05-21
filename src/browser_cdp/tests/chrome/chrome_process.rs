#[cfg(unix)]
#[test]
fn chrome_launch_retries_after_early_startup_exit() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::time::Duration;

    use super::super::super::chrome_process::ChromeProcess;
    use super::super::{EnvVarGuard, ENV_LOCK};

    let _env_lock = ENV_LOCK.lock().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let profile = temp.path().join("profile");
    fs::create_dir_all(&profile).unwrap();
    let fake_chrome = temp.path().join("fake-chrome");
    let state_path = PathBuf::from(format!("{}.count", fake_chrome.display()));
    fs::write(
        &fake_chrome,
        "#!/bin/sh\n\
         state=\"$0.count\"\n\
         count=\"$(cat \"$state\" 2>/dev/null || echo 0)\"\n\
         count=$((count + 1))\n\
         printf '%s\\n' \"$count\" > \"$state\"\n\
         if [ \"$count\" -lt 2 ]; then\n\
           echo 'transient Chrome startup failure' >&2\n\
           exit 1\n\
         fi\n\
         user_data_dir=''\n\
         for arg in \"$@\"; do\n\
           case \"$arg\" in\n\
             --user-data-dir=*) user_data_dir=\"${arg#--user-data-dir=}\" ;;\n\
           esac\n\
         done\n\
         if [ -z \"$user_data_dir\" ]; then\n\
           echo 'missing user data dir' >&2\n\
           exit 2\n\
         fi\n\
         printf '49152\\n/devtools/browser/retry\\n' > \"$user_data_dir/DevToolsActivePort\"\n\
         sleep 60\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_chrome).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&fake_chrome, permissions).unwrap();

    let _env_guard = EnvVarGuard::set("AGET_CHROME_COMMAND", fake_chrome.as_os_str());
    let chrome = ChromeProcess::launch_profile(
        &profile,
        None,
        false,
        Duration::from_secs(2),
        "owned Chrome retry test",
    )
    .unwrap();

    assert_eq!(chrome.ws_url, "ws://127.0.0.1:49152/devtools/browser/retry");
    assert_eq!(fs::read_to_string(state_path).unwrap().trim(), "2");
}
