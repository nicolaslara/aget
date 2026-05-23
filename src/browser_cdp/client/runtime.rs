use serde_json::Value;

use crate::error::{AgetError, ErrorCode};

pub(in crate::browser_cdp) fn fail_on_runtime_evaluation_exception(
    result: &Value,
) -> Result<(), AgetError> {
    if let Some(message) = runtime_evaluation_exception(result) {
        return Err(AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: format!("owned browser fallback Runtime.evaluate failed: {message}"),
        });
    }
    Ok(())
}

fn runtime_evaluation_exception(result: &Value) -> Option<String> {
    let details = result.get("exceptionDetails")?;
    let message = details
        .get("exception")
        .and_then(|exception| exception.get("description"))
        .and_then(Value::as_str)
        .or_else(|| details.get("text").and_then(Value::as_str))
        .unwrap_or("unknown Runtime.evaluate exception");
    Some(message.to_string())
}
