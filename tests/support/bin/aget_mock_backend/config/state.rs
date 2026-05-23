use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::args::Args;

pub(crate) fn assert_expected_state(args: &Args, config: &Value) -> Result<(), String> {
    if config.get("expect_state").is_none() && config.get("expect_state_cookies").is_none() {
        return Ok(());
    }
    let state = read_state(&args.state)?;
    if let Some(expected) = config.get("expect_state") {
        if &state != expected {
            return Err(format!("expected state {expected}, got {state}"));
        }
    }
    if let Some(expected) = config["expect_state_cookies"].as_array() {
        let mut actual = state["cookies"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|cookie| {
                serde_json::json!([
                    cookie["name"].as_str().unwrap_or_default(),
                    cookie["value"].as_str().unwrap_or_default()
                ])
            })
            .collect::<Vec<_>>();
        actual.sort_by_key(|value| value.to_string());
        let mut expected = expected.clone();
        expected.sort_by_key(|value| value.to_string());
        if actual != expected {
            return Err(format!(
                "expected state cookies {expected:?}, got {actual:?}"
            ));
        }
    }
    Ok(())
}

pub(crate) fn read_state(path: &Path) -> Result<Value, String> {
    serde_json::from_str(
        &fs::read_to_string(path).map_err(|error| format!("read state: {error}"))?,
    )
    .map_err(|error| format!("parse state: {error}"))
}

pub(crate) fn expand_state_placeholders(template: &str, state: &Value) -> String {
    template
        .replace(
            "{first_cookie_value}",
            first_cookie_value(state).unwrap_or_default().as_str(),
        )
        .replace(
            "{first_storage_value}",
            first_storage_value(state).unwrap_or_default().as_str(),
        )
}

fn first_cookie_value(state: &Value) -> Option<String> {
    state["cookies"].as_array()?.first()?["value"]
        .as_str()
        .map(ToOwned::to_owned)
}

fn first_storage_value(state: &Value) -> Option<String> {
    state["origins"].as_array()?.first()?["localStorage"]
        .as_array()?
        .first()?["value"]
        .as_str()
        .map(ToOwned::to_owned)
}
