use serde_json::Value;

pub(in crate::browser_cdp) fn preferred_page_target_id(target_infos: &[Value]) -> Option<String> {
    let mut fallback = None;
    for target in target_infos {
        let Some((target_id, url)) = trackable_page_target(target) else {
            continue;
        };
        fallback.get_or_insert_with(|| target_id.to_string());
        if !url.is_empty() && url != "about:blank" {
            return Some(target_id.to_string());
        }
    }
    fallback
}

fn trackable_page_target(target: &Value) -> Option<(&str, &str)> {
    let target_type = target.get("type").and_then(Value::as_str)?;
    if target_type != "page" && target_type != "webview" {
        return None;
    }
    let url = target
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if is_internal_chrome_target(url) {
        return None;
    }
    let target_id = target.get("targetId").and_then(Value::as_str)?;
    Some((target_id, url))
}

fn is_internal_chrome_target(url: &str) -> bool {
    url.starts_with("chrome://")
        || url.starts_with("chrome-extension://")
        || url.starts_with("devtools://")
}
