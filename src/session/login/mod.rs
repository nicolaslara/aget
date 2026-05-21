mod compat;
mod owned;
mod pending;
mod session_merge;
mod types;

#[cfg(test)]
mod tests;

use crate::error::{AgetError, ErrorCode};

pub use self::compat::{cancel_login_session, finish_login_session, start_login_session};
pub use self::pending::complete_login_session;
pub use self::session_merge::merge_login_session;
pub use self::types::{
    LoginCancelOptions, LoginCancelResult, LoginCompleteOptions, LoginFinishOptions,
    LoginFinishResult, LoginStartOptions, LoginStartResult, PendingLogin,
};

pub(crate) use self::owned::{
    cancel_owned_login_session, finish_owned_login_session, start_owned_login_session,
};

fn io_aget_error(error: impl std::fmt::Display) -> AgetError {
    AgetError::Stable {
        code: ErrorCode::IoError,
        message: error.to_string(),
    }
}
