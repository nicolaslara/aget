use crate::error::AgetError;

use super::super::io_aget_error;

pub(in crate::browser_cdp) fn origin_storage_expression() -> &'static str {
    r#"(() => {
        const result = { origin: location.origin, localStorage: [], sessionStorage: [] };
        try {
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);
                result.localStorage.push({ name: key, value: localStorage.getItem(key) });
            }
        } catch(e) {}
        try {
            for (let i = 0; i < sessionStorage.length; i++) {
                const key = sessionStorage.key(i);
                result.sessionStorage.push({ name: key, value: sessionStorage.getItem(key) });
            }
        } catch(e) {}
        return result;
    })()"#
}

pub(in crate::browser_cdp) fn local_storage_set_expression(
    name: &str,
    value: &str,
) -> Result<String, AgetError> {
    Ok(format!(
        "localStorage.setItem({}, {})",
        serde_json::to_string(name).map_err(io_aget_error)?,
        serde_json::to_string(value).map_err(io_aget_error)?
    ))
}

pub(in crate::browser_cdp) fn session_storage_set_expression(
    name: &str,
    value: &str,
) -> Result<String, AgetError> {
    Ok(format!(
        "sessionStorage.setItem({}, {})",
        serde_json::to_string(name).map_err(io_aget_error)?,
        serde_json::to_string(value).map_err(io_aget_error)?
    ))
}
