use std::env;
use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::args::Args;
use crate::output::print_backend_result;

pub(crate) fn read_config() -> Result<Value, String> {
    let config_path = env::current_exe()
        .map_err(|error| format!("locate mock backend executable: {error}"))?
        .with_extension("json");
    if !config_path.exists() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(
        &fs::read_to_string(&config_path)
            .map_err(|error| format!("read mock backend config: {error}"))?,
    )
    .map_err(|error| format!("parse mock backend config: {error}"))
}

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

fn validate_crawl4ai_options(args: &Args) -> Result<(), String> {
    const ALLOWED: &[&str] = &[
        "base_url",
        "target_elements",
        "excluded_selector",
        "excluded_tags",
        "exclude_all_images",
        "exclude_domains",
        "exclude_external_images",
        "exclude_external_links",
        "exclude_internal_links",
        "exclude_social_media_domains",
        "exclude_social_media_links",
        "only_text",
        "process_iframes",
        "remove_consent_popups",
        "remove_forms",
        "remove_overlay_elements",
        "keep_data_attributes",
        "word_count_threshold",
        "wait_until",
        "page_timeout",
        "wait_for_timeout",
        "delay_before_return_html",
        "wait_for_images",
        "scan_full_page",
        "scroll_delay",
        "max_scroll_steps",
        "flatten_shadow_dom",
        "body_width",
        "bypass_tables",
        "close_quote",
        "default_image_alt",
        "emphasis_mark",
        "escape_backslash",
        "escape_dash",
        "escape_dot",
        "escape_plus",
        "escape_snob",
        "google_doc",
        "google_list_indent",
        "handle_code_in_pre",
        "hide_strikethrough",
        "ignore_anchors",
        "ignore_emphasis",
        "ignore_images",
        "images_as_html",
        "images_to_alt",
        "images_with_size",
        "ignore_links",
        "inline_links",
        "ignore_mailto_links",
        "links_each_paragraph",
        "ignore_tables",
        "include_sup_sub",
        "mark_code",
        "open_quote",
        "pad_tables",
        "preserve_tags",
        "protect_links",
        "single_line_break",
        "skip_internal_links",
        "strong_mark",
        "ul_item_mark",
        "unicode_snob",
        "use_automatic_links",
        "wrap_links",
        "wrap_list_items",
        "wrap_tables",
    ];
    for option in &args.extractor_options {
        let Some((key, _value)) = option.split_once('=') else {
            return structured_validation_error(
                args,
                format!("extractor option must use key=value form: {option}"),
            );
        };
        let key = key.strip_prefix("crawl4ai.").ok_or_else(|| {
            format!("extractor option '{key}' must use the crawl4ai.<key> namespace")
        })?;
        if !ALLOWED.contains(&key) {
            let allowed = ALLOWED.join(", ");
            return structured_validation_error(
                args,
                format!("unsupported extractor option '{key}'; supported keys: {allowed}"),
            );
        }
    }
    if let Some(wait_for) = &args.wait_for {
        let normalized = wait_for.trim().to_ascii_lowercase();
        if normalized.starts_with("js:")
            || ["=>", "function(", "return ", ";"]
                .iter()
                .any(|marker| normalized.contains(marker))
        {
            return structured_validation_error(
                args,
                "--wait-for-selector only supports CSS selectors in v1; JavaScript wait conditions are not allowed".to_string(),
            );
        }
    }
    Ok(())
}

fn structured_validation_error(args: &Args, message: String) -> Result<(), String> {
    print_backend_result(false, None, None, Vec::new(), Some(message.clone()));
    fs::write(
        &args.metadata,
        serde_json::to_vec_pretty(&serde_json::json!({
            "ok": false,
            "final_url": null,
            "content": null,
            "warnings": [],
            "error": message,
        }))
        .map_err(|error| format!("serialize validation metadata: {error}"))?,
    )
    .map_err(|error| format!("write validation metadata: {error}"))?;
    std::process::exit(1);
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
