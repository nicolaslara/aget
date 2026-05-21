mod compat;
mod owned;
mod profile;

#[cfg(test)]
mod tests;

use std::path::PathBuf;

use crate::error::{AgetError, ErrorCode};

pub use self::compat::import_chrome_session;
pub(crate) use self::owned::import_owned_chrome_session;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChromeImportOptions {
    pub profile: String,
    pub name: String,
    pub domains: Vec<String>,
    pub tmp_dir: PathBuf,
}

fn io_aget_error(error: impl std::fmt::Display) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}
