use aget::{
    ErrorCode, ErrorResponse, GetSuccess, InlineContent, TimingMs, ENVELOPE_SCHEMA_VERSION,
};
use serde::Serialize;
use serde_json::Value;

pub(crate) fn print_success_envelope(
    command: &str,
    data: Value,
    warnings: Vec<String>,
    timing_ms: TimingMs,
) -> Result<(), ErrorResponse> {
    let envelope = serde_json::json!({
        "ok": true,
        "schema_version": ENVELOPE_SCHEMA_VERSION,
        "command": command,
        "data": data,
        "warnings": warnings,
        "timing_ms": timing_ms,
    });
    println!("{}", serde_json::to_string(&envelope).map_err(io_error)?);
    Ok(())
}

pub(crate) fn get_envelope_data(
    success: &GetSuccess,
    inline_content: InlineContent,
) -> Result<Value, ErrorResponse> {
    let mut data = envelope_data(success, &["ok", "warnings", "timing_ms"])?;
    if let Value::Object(object) = &mut data {
        let include_content = match inline_content {
            InlineContent::Always => true,
            InlineContent::Never => false,
            InlineContent::Auto => !success.sensitive,
        };
        if !include_content {
            object.remove("content");
        }
    }
    Ok(data)
}

pub(crate) fn envelope_data<T: Serialize>(
    value: &T,
    remove_keys: &[&str],
) -> Result<Value, ErrorResponse> {
    let mut data = serde_json::to_value(value).map_err(io_error)?;
    if let Value::Object(object) = &mut data {
        for key in remove_keys {
            object.remove(*key);
        }
    }
    Ok(data)
}

pub(crate) fn io_error(error: impl std::fmt::Display) -> ErrorResponse {
    ErrorResponse::new(ErrorCode::IoError, error.to_string())
}

pub(crate) fn error_response(error: aget::AgetError) -> ErrorResponse {
    match error {
        aget::AgetError::Stable { code, message } => ErrorResponse::new(code, message),
    }
}
