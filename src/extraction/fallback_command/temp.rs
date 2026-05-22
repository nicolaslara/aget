use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AgetError;

use super::super::{create_private_dir, create_private_file, io_aget_error};

pub(super) struct TempAgentBrowserProfile {
    path: PathBuf,
}

impl TempAgentBrowserProfile {
    pub(super) fn new(tmp_dir: &Path) -> Result<Self, AgetError> {
        fs::create_dir_all(tmp_dir).map_err(io_aget_error)?;
        let path = tmp_dir
            .join("agent-browser")
            .join(super::unique_agent_browser_session_name());
        create_private_dir(&path).map_err(io_aget_error)?;
        Ok(Self { path })
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempAgentBrowserProfile {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(super) struct TempOutputFile {
    path: PathBuf,
}

impl TempOutputFile {
    pub(super) fn new(dir: &Path, prefix: &str) -> Result<Self, AgetError> {
        fs::create_dir_all(dir).map_err(io_aget_error)?;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        Ok(Self {
            path: dir.join(format!("{prefix}-{}-{nanos}.txt", std::process::id())),
        })
    }

    pub(super) fn create(&self) -> Result<fs::File, AgetError> {
        create_private_file(&self.path).map_err(io_aget_error)
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempOutputFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
