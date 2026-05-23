use std::fs;

use crate::args::Args;
use crate::output::print_backend_result;

pub(crate) fn validate_crawl4ai_options(args: &Args) -> Result<(), String> {
    const ALLOWED: &[&str] = &[
        "base_url",
        "cache",
        "cache_mode",
        "css_selector",
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
        "keep_attrs",
        "keep_data_attributes",
        "prettiify",
        "process_in_browser",
        "user_agent",
        "locale",
        "timezone_id",
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
