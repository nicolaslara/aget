use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    UsageError,
    BackendUnavailable,
    Timeout,
    RequiresUserAction,
    AuthFailed,
    ExtractionFailed,
    SessionConflict,
    PrivacyPolicyBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub ok: bool,
    pub error: ErrorBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<String>,
}

impl ErrorResponse {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: ErrorBody {
                code,
                message: message.into(),
                retry: None,
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum AgetError {
    #[error("{message}")]
    Stable { code: ErrorCode, message: String },
}

impl AgetError {
    pub fn code(&self) -> ErrorCode {
        match self {
            AgetError::Stable { code, .. } => *code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_error_code_as_stable_snake_case() {
        let code = serde_json::to_string(&ErrorCode::RequiresUserAction).unwrap();

        assert_eq!(code, "\"requires_user_action\"");
    }

    #[test]
    fn serializes_error_response_shape() {
        let response = ErrorResponse::new(ErrorCode::BackendUnavailable, "cmux not found");
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "backend_unavailable");
        assert_eq!(value["error"]["message"], "cmux not found");
    }
}
