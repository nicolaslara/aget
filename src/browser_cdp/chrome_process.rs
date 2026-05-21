use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::error::{AgetError, ErrorCode};
use crate::process::{configure_local_command, create_private_file, TempOutputFile};

use super::discovery::{classify_chrome_startup_error, wait_for_devtools_active_port};
use super::process::{configure_chrome_process_group, terminate_child};
use super::{create_private_dir, io_aget_error};

const CHROME_LAUNCH_ATTEMPTS: usize = 3;
const CHROME_LAUNCH_RETRY_DELAY: Duration = Duration::from_millis(500);

pub(super) struct ChromeProcess {
    child: Child,
    pub(super) ws_url: String,
    user_data_dir: PathBuf,
    remove_user_data_dir: bool,
    terminated: bool,
    _stderr_capture: TempOutputFile,
}

impl ChromeProcess {
    pub(super) fn launch_temp(
        tmp_dir: &Path,
        timeout: Duration,
        operation: &'static str,
    ) -> Result<Self, AgetError> {
        let user_data_dir = unique_profile_dir(tmp_dir)?;
        Self::launch_with_user_data_dir(
            user_data_dir,
            true,
            None,
            false,
            true,
            None,
            timeout,
            operation,
        )
    }

    pub(super) fn launch_profile(
        user_data_dir: &Path,
        profile_directory: Option<&str>,
        use_real_keychain: bool,
        timeout: Duration,
        operation: &'static str,
    ) -> Result<Self, AgetError> {
        Self::launch_with_user_data_dir(
            user_data_dir.to_path_buf(),
            false,
            profile_directory,
            use_real_keychain,
            true,
            None,
            timeout,
            operation,
        )
    }

    pub(super) fn launch_login(
        user_data_dir: &Path,
        url: &str,
        timeout: Duration,
        operation: &'static str,
    ) -> Result<Self, AgetError> {
        Self::launch_with_user_data_dir(
            user_data_dir.to_path_buf(),
            false,
            None,
            false,
            false,
            Some(url),
            timeout,
            operation,
        )
    }

    fn launch_with_user_data_dir(
        user_data_dir: PathBuf,
        remove_user_data_dir: bool,
        profile_directory: Option<&str>,
        use_real_keychain: bool,
        headless: bool,
        startup_url: Option<&str>,
        timeout: Duration,
        operation: &'static str,
    ) -> Result<Self, AgetError> {
        let executable = find_chrome_binary().ok_or_else(|| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!(
                "{operation} could not find Chrome; set AGET_CHROME_COMMAND to a Chrome/Chromium executable"
            ),
        })?;
        let stderr_capture =
            TempOutputFile::new(&env::temp_dir(), "aget-chrome-stderr").map_err(io_aget_error)?;
        let mut last_error = None;
        for attempt in 1..=CHROME_LAUNCH_ATTEMPTS {
            let stderr = create_private_file(stderr_capture.path()).map_err(io_aget_error)?;
            let mut command = chrome_launch_command(
                &executable,
                &user_data_dir,
                profile_directory,
                use_real_keychain,
                headless,
                startup_url,
                stderr,
            );
            let _ = fs::remove_file(user_data_dir.join("DevToolsActivePort"));
            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(error) => {
                    last_error = Some(AgetError::Stable {
                        code: ErrorCode::BackendUnavailable,
                        message: format!(
                            "{operation} could not launch Chrome at '{}': {error}",
                            executable.display()
                        ),
                    });
                    if attempt < CHROME_LAUNCH_ATTEMPTS {
                        thread::sleep(CHROME_LAUNCH_RETRY_DELAY);
                    }
                    continue;
                }
            };

            match wait_for_devtools_active_port(
                &mut child,
                &user_data_dir,
                &stderr_capture,
                timeout,
            ) {
                Ok(ws_url) => {
                    return Ok(Self {
                        child,
                        ws_url,
                        user_data_dir,
                        remove_user_data_dir,
                        terminated: false,
                        _stderr_capture: stderr_capture,
                    });
                }
                Err(error) => {
                    terminate_child(&mut child);
                    last_error = Some(classify_chrome_startup_error(
                        operation,
                        error,
                        &stderr_capture,
                    ));
                    if attempt < CHROME_LAUNCH_ATTEMPTS {
                        thread::sleep(CHROME_LAUNCH_RETRY_DELAY);
                    }
                }
            }
        }

        if remove_user_data_dir {
            let _ = fs::remove_dir_all(&user_data_dir);
        }
        Err(last_error.unwrap_or_else(|| AgetError::Stable {
            code: ErrorCode::BackendUnavailable,
            message: format!("{operation} could not launch Chrome"),
        }))
    }

    pub(super) fn detach(mut self) {
        self.terminated = true;
    }

    pub(super) fn id(&self) -> u32 {
        self.child.id()
    }

    pub(super) fn wait_or_kill(&mut self, timeout: Duration) {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match self.child.try_wait() {
                Ok(Some(_)) => {
                    self.terminated = true;
                    return;
                }
                Ok(None) => thread::sleep(Duration::from_millis(25)),
                Err(_) => {
                    self.terminated = true;
                    return;
                }
            }
        }
        terminate_child(&mut self.child);
        self.terminated = true;
    }
}

impl Drop for ChromeProcess {
    fn drop(&mut self) {
        if !self.terminated {
            terminate_child(&mut self.child);
            self.terminated = true;
        }
        if self.remove_user_data_dir {
            let _ = fs::remove_dir_all(&self.user_data_dir);
        }
    }
}

fn chrome_launch_command(
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

fn unique_profile_dir(tmp_dir: &Path) -> Result<PathBuf, AgetError> {
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

fn find_chrome_binary() -> Option<PathBuf> {
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

#[cfg(test)]
mod tests;
