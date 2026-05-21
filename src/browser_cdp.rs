mod chrome_process;
mod client;
mod discovery;
mod login;
mod page_scripts;
mod process;
mod render;
mod session_data;
mod state;

use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

pub(crate) use self::login::{
    close_login_browser, export_login_browser_state, start_login_browser, BrowserLoginCloseRequest,
    BrowserLoginStartRequest, BrowserLoginStateExportRequest,
};
pub(crate) use self::render::{render_page, BrowserRenderRequest, PageWaitUntil};
pub(crate) use self::state::{export_browser_state, BrowserStateExportRequest};
use crate::error::{AgetError, ErrorCode};

const CHROME_SHUTDOWN_WAIT: Duration = Duration::from_secs(1);
const CHROME_SANDBOX_STARTUP_HINT: &str = "Hint: Chrome sandbox/namespace startup failure; in containers or VMs, set AGET_CHROME_COMMAND to a Chrome/Chromium executable that can run with --no-sandbox";
const CHROME_SILENT_STARTUP_HINT: &str = "Hint: Chrome exited without startup diagnostics; in containers or VMs, set AGET_CHROME_COMMAND to a Chrome/Chromium wrapper that can run with --no-sandbox";

fn io_aget_error(error: impl ToString) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}

fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    set_private_dir_permissions(path)
}

#[cfg(unix)]
fn set_private_dir_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
fn set_private_dir_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests;
