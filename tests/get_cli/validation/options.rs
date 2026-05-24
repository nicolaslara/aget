use assert_cmd::Command;

const FORWARDED_AGET_OPTIONS: &[&str] = &[
    "aget.base_url=https://docs.example/raw/",
    "aget.cache=bypass",
    "aget.cache_mode=disabled",
    "aget.scan_full_page=true",
    "aget.scroll_delay=0.3",
    "aget.max_scroll_steps=5",
    "aget.flatten_shadow_dom=true",
    "aget.process_iframes=true",
    "aget.css_selector=main.article",
    "aget.remove_forms=true",
    "aget.keep_data_attributes=true",
    "aget.exclude_all_images=true",
    "aget.exclude_domains=external.example,cdn.example",
    "aget.exclude_external_images=true",
    "aget.exclude_external_links=true",
    "aget.exclude_internal_links=true",
    "aget.exclude_social_media_domains=social.example,community.example",
    "aget.exclude_social_media_links=true",
    "aget.excluded_selector=aside,.promo",
    "aget.remove_consent_popups=true",
    "aget.remove_overlay_elements=false",
    "aget.keep_attrs=aria-label,rel",
    "aget.prettiify=true",
    "aget.process_in_browser=false",
    "aget.user_agent=aget-test/2.0",
    "aget.locale=sv-SE",
    "aget.timezone_id=Europe/Stockholm",
    "aget.body_width=40",
    "aget.bypass_tables=true",
    "aget.close_quote=>>",
    "aget.default_image_alt=Missing alt",
    "aget.emphasis_mark=*",
    "aget.escape_backslash=false",
    "aget.escape_dash=false",
    "aget.escape_dot=false",
    "aget.escape_plus=false",
    "aget.escape_snob=true",
    "aget.google_doc=true",
    "aget.google_list_indent=72",
    "aget.handle_code_in_pre=true",
    "aget.hide_strikethrough=true",
    "aget.ignore_anchors=true",
    "aget.ignore_emphasis=true",
    "aget.ignore_images=true",
    "aget.images_as_html=true",
    "aget.images_to_alt=true",
    "aget.images_with_size=true",
    "aget.ignore_links=true",
    "aget.inline_links=false",
    "aget.ignore_mailto_links=false",
    "aget.links_each_paragraph=true",
    "aget.ignore_tables=true",
    "aget.include_sup_sub=true",
    "aget.mark_code=false",
    "aget.open_quote=<<",
    "aget.pad_tables=true",
    "aget.preserve_tags=custom-card,math-box",
    "aget.protect_links=true",
    "aget.single_line_break=true",
    "aget.skip_internal_links=true",
    "aget.strong_mark=__",
    "aget.ul_item_mark=-",
    "aget.unicode_snob=false",
    "aget.use_automatic_links=false",
    "aget.wrap_links=false",
    "aget.wrap_list_items=true",
    "aget.wrap_tables=true",
];

#[test]
fn get_owned_extractor_accepts_supported_backend_options() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");

    let mut args = vec![
        "--envelope",
        "json",
        "get",
        "raw:<main><a href=\"/docs\">Docs</a><img src=\"/logo.png\"></main>",
    ];
    for option in FORWARDED_AGET_OPTIONS {
        args.push("--backend-option");
        args.push(option);
    }

    let mut cmd = Command::cargo_bin("aget").unwrap();
    cmd.env("AGET_HOME", &aget_home)
        .args(args)
        .assert()
        .success();
}
