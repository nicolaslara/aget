use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AgetError;
use crate::process::configure_local_command;

use super::super::process::configure_chrome_process_group;
use super::super::{create_private_dir, io_aget_error};

pub(super) fn chrome_launch_command(
    executable: &Path,
    user_data_dir: &Path,
    profile_directory: Option<&str>,
    use_real_keychain: bool,
    headless: bool,
    startup_url: Option<&str>,
    stderr: fs::File,
) -> Command {
    let mut command = Command::new(executable);
    command
        .arg("--remote-debugging-port=0")
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg("--disable-background-networking")
        .arg("--disable-backgrounding-occluded-windows")
        .arg("--disable-component-update")
        .arg("--disable-default-apps")
        .arg("--disable-hang-monitor")
        .arg("--disable-popup-blocking")
        .arg("--disable-prompt-on-repost")
        .arg("--disable-sync")
        .arg("--disable-features=Translate")
        .arg("--enable-features=NetworkService,NetworkServiceInProcess")
        .arg("--metrics-recording-only")
        .arg(format!("--user-data-dir={}", user_data_dir.display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr));
    if headless {
        command
            .arg("--window-size=1280,720")
            .arg("--headless=new")
            .arg("--enable-unsafe-swiftshader");
    }
    if !use_real_keychain {
        command
            .arg("--password-store=basic")
            .arg("--use-mock-keychain");
    }
    if let Some(profile_directory) = profile_directory {
        command.arg(format!("--profile-directory={profile_directory}"));
    }
    if cfg!(target_os = "linux") {
        command.arg("--no-sandbox").arg("--disable-dev-shm-usage");
    }
    if let Some(startup_url) = startup_url {
        command.arg("--new-window").arg(startup_url);
    }
    configure_local_command(&mut command);
    configure_chrome_process_group(&mut command);
    command
}

pub(super) fn unique_profile_dir(tmp_dir: &Path) -> Result<PathBuf, AgetError> {
    let owned_dir = tmp_dir.join("owned-chrome");
    create_private_dir(&owned_dir).map_err(io_aget_error)?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let path = owned_dir.join(format!("aget-chrome-{}-{nanos}", std::process::id()));
    create_private_dir(&path).map_err(io_aget_error)?;
    Ok(path)
}

pub(super) fn find_chrome_binary() -> Option<PathBuf> {
    if let Some(command) = env::var_os("AGET_CHROME_COMMAND") {
        if !command.is_empty() {
            return Some(PathBuf::from(command));
        }
    }

    for path in platform_chrome_candidates() {
        if path.is_file() {
            return Some(path);
        }
    }

    for command in [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "chrome",
        "brave-browser",
        "brave-browser-stable",
    ] {
        if let Some(path) = find_on_path(command) {
            return Some(path);
        }
    }

    None
}

fn platform_chrome_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    #[cfg(target_os = "macos")]
    {
        candidates.extend([
            PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
            PathBuf::from(
                "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
            ),
            PathBuf::from("/Applications/Chromium.app/Contents/MacOS/Chromium"),
            PathBuf::from("/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
        ]);
        if let Some(home) = env::var_os("HOME") {
            let home = PathBuf::from(home);
            candidates.extend([
                home.join("Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
                home.join("Applications/Chromium.app/Contents/MacOS/Chromium"),
                home.join("Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
            ]);
        }
    }

    #[cfg(target_os = "windows")]
    {
        candidates.extend([
            PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
            PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"),
        ]);
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            let local = PathBuf::from(local);
            candidates.extend([
                local.join(r"Google\Chrome\Application\chrome.exe"),
                local.join(r"BraveSoftware\Brave-Browser\Application\brave.exe"),
            ]);
        }
    }

    candidates
}

fn find_on_path(command: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(command))
        .find(|path| path.is_file())
}
