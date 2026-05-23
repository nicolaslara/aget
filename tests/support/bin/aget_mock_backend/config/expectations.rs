use std::env;

use serde_json::Value;

use crate::args::Args;

use super::options::validate_crawl4ai_options;

pub(crate) fn assert_expected_args(args: &Args, config: &Value) -> Result<(), String> {
    for (field, actual) in [
        ("format", Some(args.format.as_str())),
        ("selector", args.selector.as_deref()),
        ("exclude_selector", args.exclude_selector.as_deref()),
        ("wait_for", args.wait_for.as_deref()),
    ] {
        if let Some(expected) = config[format!("expect_{field}")].as_str() {
            if Some(expected) != actual {
                return Err(format!("expected {field}={expected:?}, got {actual:?}"));
            }
        }
    }
    if let Some(expected) = config["expect_extractor_options"].as_array() {
        let expected = expected
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .ok_or_else(|| "expect_extractor_options entries must be strings".to_string())
                    .map(ToOwned::to_owned)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if args.extractor_options != expected {
            return Err(format!(
                "expected extractor_options={expected:?}, got {:?}",
                args.extractor_options
            ));
        }
    }
    if config["validate_crawl4ai_options"].as_bool().unwrap_or(false) {
        validate_crawl4ai_options(args)?;
    }
    Ok(())
}

pub(crate) fn assert_expected_environment(config: &Value) -> Result<(), String> {
    for key in config["expect_env_absent"].as_array().into_iter().flatten() {
        let key = key
            .as_str()
            .ok_or_else(|| "expect_env_absent entries must be strings".to_string())?;
        if env::var_os(key).is_some() {
            return Err(format!("expected environment variable {key} to be absent"));
        }
    }
    Ok(())
}
