mod launch;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{AgetError, ErrorCode};
use crate::process::{create_private_file, TempOutputFile};

use super::discovery::{classify_chrome_startup_error, wait_for_devtools_active_port};
use super::io_aget_error;
use super::process::terminate_child;
use launch::{chrome_launch_command, find_chrome_binary, unique_profile_dir};

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

#[cfg(test)]
mod tests;
