use std::fs;

use assert_cmd::Command;
use serde_json::json;

use crate::support::get_cli::{metadata_files, mock_backend_command};

#[test]
fn get_command_backend_accepts_scan_full_page_options() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({
            "behavior": "success",
            "validate_crawl4ai_options": true,
            "expect_extractor_options": [
                "crawl4ai.base_url=https://docs.example/raw/",
                "crawl4ai.scan_full_page=true",
                "crawl4ai.scroll_delay=0.3",
                "crawl4ai.max_scroll_steps=5",
                "crawl4ai.flatten_shadow_dom=true",
                "crawl4ai.process_iframes=true",
                "crawl4ai.remove_forms=true",
                "crawl4ai.keep_data_attributes=true",
                "crawl4ai.exclude_all_images=true",
                "crawl4ai.exclude_domains=external.example,cdn.example",
                "crawl4ai.exclude_external_images=true",
                "crawl4ai.exclude_external_links=true",
                "crawl4ai.exclude_internal_links=true",
                "crawl4ai.exclude_social_media_domains=social.example,community.example",
                "crawl4ai.exclude_social_media_links=true",
                "crawl4ai.remove_overlay_elements=false",
                "crawl4ai.body_width=40",
                "crawl4ai.bypass_tables=true",
                "crawl4ai.close_quote=>>",
                "crawl4ai.default_image_alt=Missing alt",
                "crawl4ai.emphasis_mark=*",
                "crawl4ai.escape_snob=true",
                "crawl4ai.hide_strikethrough=true",
                "crawl4ai.ignore_emphasis=true",
                "crawl4ai.ignore_images=true",
                "crawl4ai.images_as_html=true",
                "crawl4ai.images_to_alt=true",
                "crawl4ai.images_with_size=true",
                "crawl4ai.ignore_links=true",
                "crawl4ai.inline_links=false",
                "crawl4ai.ignore_mailto_links=false",
                "crawl4ai.links_each_paragraph=true",
                "crawl4ai.ignore_tables=true",
                "crawl4ai.include_sup_sub=true",
                "crawl4ai.mark_code=false",
                "crawl4ai.open_quote=<<",
                "crawl4ai.pad_tables=true",
                "crawl4ai.protect_links=true",
                "crawl4ai.single_line_break=true",
                "crawl4ai.skip_internal_links=true",
                "crawl4ai.strong_mark=__",
                "crawl4ai.ul_item_mark=-",
                "crawl4ai.unicode_snob=false",
                "crawl4ai.use_automatic_links=false",
                "crawl4ai.wrap_links=false",
                "crawl4ai.wrap_list_items=true",
                "crawl4ai.wrap_tables=true"
            ]
        }),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    cmd.env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/scan-options",
            "--backend-option",
            "crawl4ai.base_url=https://docs.example/raw/",
            "--backend-option",
            "crawl4ai.scan_full_page=true",
            "--backend-option",
            "crawl4ai.scroll_delay=0.3",
            "--backend-option",
            "crawl4ai.max_scroll_steps=5",
            "--backend-option",
            "crawl4ai.flatten_shadow_dom=true",
            "--backend-option",
            "crawl4ai.process_iframes=true",
            "--backend-option",
            "crawl4ai.remove_forms=true",
            "--backend-option",
            "crawl4ai.keep_data_attributes=true",
            "--backend-option",
            "crawl4ai.exclude_all_images=true",
            "--backend-option",
            "crawl4ai.exclude_domains=external.example,cdn.example",
            "--backend-option",
            "crawl4ai.exclude_external_images=true",
            "--backend-option",
            "crawl4ai.exclude_external_links=true",
            "--backend-option",
            "crawl4ai.exclude_internal_links=true",
            "--backend-option",
            "crawl4ai.exclude_social_media_domains=social.example,community.example",
            "--backend-option",
            "crawl4ai.exclude_social_media_links=true",
            "--backend-option",
            "crawl4ai.remove_overlay_elements=false",
            "--backend-option",
            "crawl4ai.body_width=40",
            "--backend-option",
            "crawl4ai.bypass_tables=true",
            "--backend-option",
            "crawl4ai.close_quote=>>",
            "--backend-option",
            "crawl4ai.default_image_alt=Missing alt",
            "--backend-option",
            "crawl4ai.emphasis_mark=*",
            "--backend-option",
            "crawl4ai.escape_snob=true",
            "--backend-option",
            "crawl4ai.hide_strikethrough=true",
            "--backend-option",
            "crawl4ai.ignore_emphasis=true",
            "--backend-option",
            "crawl4ai.ignore_images=true",
            "--backend-option",
            "crawl4ai.images_as_html=true",
            "--backend-option",
            "crawl4ai.images_to_alt=true",
            "--backend-option",
            "crawl4ai.images_with_size=true",
            "--backend-option",
            "crawl4ai.ignore_links=true",
            "--backend-option",
            "crawl4ai.inline_links=false",
            "--backend-option",
            "crawl4ai.ignore_mailto_links=false",
            "--backend-option",
            "crawl4ai.links_each_paragraph=true",
            "--backend-option",
            "crawl4ai.ignore_tables=true",
            "--backend-option",
            "crawl4ai.include_sup_sub=true",
            "--backend-option",
            "crawl4ai.mark_code=false",
            "--backend-option",
            "crawl4ai.open_quote=<<",
            "--backend-option",
            "crawl4ai.pad_tables=true",
            "--backend-option",
            "crawl4ai.protect_links=true",
            "--backend-option",
            "crawl4ai.single_line_break=true",
            "--backend-option",
            "crawl4ai.skip_internal_links=true",
            "--backend-option",
            "crawl4ai.strong_mark=__",
            "--backend-option",
            "crawl4ai.ul_item_mark=-",
            "--backend-option",
            "crawl4ai.unicode_snob=false",
            "--backend-option",
            "crawl4ai.use_automatic_links=false",
            "--backend-option",
            "crawl4ai.wrap_links=false",
            "--backend-option",
            "crawl4ai.wrap_list_items=true",
            "--backend-option",
            "crawl4ai.wrap_tables=true",
        ])
        .assert()
        .success();
}

#[test]
fn get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "success", "validate_crawl4ai_options": true}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/unsupported-option",
            "--backend-option",
            "crawl4ai.js_code=alert(1)",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unsupported extractor option 'js_code'"));

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["ok"], false);
    assert!(metadata["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unsupported extractor option 'js_code'"));
}

#[test]
fn get_real_helper_rejects_javascript_wait_before_crawl4ai_import() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command(
        temp.path(),
        json!({"behavior": "success", "validate_crawl4ai_options": true}),
    );

    let mut cmd = Command::cargo_bin("aget").unwrap();
    let output = cmd
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "https://example.com/js-wait",
            "--wait-for-selector",
            "js:() => true",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["error"]["code"], "extraction_failed");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("--wait-for-selector only supports CSS selectors in v1"));

    let metadata_files = metadata_files(&aget_home);
    assert_eq!(metadata_files.len(), 1);
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata_files[0]).unwrap()).unwrap();
    assert_eq!(metadata["ok"], false);
    assert!(metadata["error"]["message"]
        .as_str()
        .unwrap()
        .contains("JavaScript wait conditions are not allowed"));
}
