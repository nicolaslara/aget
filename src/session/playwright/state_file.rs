use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::PlaywrightState;

#[derive(Debug)]
pub struct TempStateFile {
    path: PathBuf,
}

static STATE_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

impl TempStateFile {
    pub fn write(dir: &Path, state: &PlaywrightState) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let path = unique_state_path(dir);
        let mut file = create_private_file(&path)?;
        serde_json::to_writer_pretty(&mut file, state).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempStateFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn unique_state_path(dir: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let counter = STATE_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    dir.join(format!(
        "playwright-state-{}-{nanos}-{counter}.json",
        std::process::id(),
    ))
}

fn create_private_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    set_private_file_mode(&mut options);
    let file = options.open(path)?;
    set_private_file_permissions(path)?;
    Ok(file)
}

#[cfg(unix)]
fn set_private_file_mode(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;

    options.mode(0o600);
}

#[cfg(not(unix))]
fn set_private_file_mode(_options: &mut OpenOptions) {}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}
