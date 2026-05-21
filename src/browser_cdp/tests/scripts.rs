use serde_json::json;

use super::super::client::preferred_page_target_id;
use super::super::page_scripts::{
    full_page_scan_expression, local_storage_set_expression, rendered_overlay_cleanup_expression,
    selector_exists_expression, session_storage_set_expression,
    shadow_dom_attach_override_expression, shadow_dom_flatten_expression,
};

#[test]
fn local_storage_expression_json_quotes_key_and_value() {
    let expression = local_storage_set_expression("token\"name", "line\nvalue</script>").unwrap();

    assert_eq!(
        expression,
        "localStorage.setItem(\"token\\\"name\", \"line\\nvalue</script>\")"
    );
}

#[test]
fn session_storage_expression_json_quotes_key_and_value() {
    let expression = session_storage_set_expression("token\"name", "line\nvalue</script>").unwrap();

    assert_eq!(
        expression,
        "sessionStorage.setItem(\"token\\\"name\", \"line\\nvalue</script>\")"
    );
}

#[test]
fn prefers_existing_non_internal_page_target() {
    let target_id = preferred_page_target_id(&[
        json!({
            "targetId": "chrome",
            "type": "page",
            "url": "chrome://new-tab-page/"
        }),
        json!({
            "targetId": "blank",
            "type": "page",
            "url": "about:blank"
        }),
        json!({
            "targetId": "login",
            "type": "page",
            "url": "https://example.com/login"
        }),
    ]);

    assert_eq!(target_id.as_deref(), Some("login"));
}

#[test]
fn selector_wait_expression_json_quotes_css_selector() {
    let expression = selector_exists_expression(r#"main[data-name="a b"]"#).unwrap();

    assert_eq!(
        expression,
        r#"document.querySelector("main[data-name=\"a b\"]") !== null"#
    );
}

#[test]
fn selector_wait_expression_strips_explicit_css_prefix() {
    let expression = selector_exists_expression("css: main #ready").unwrap();

    assert_eq!(
        expression,
        r#"document.querySelector("main #ready") !== null"#
    );
}

#[test]
fn full_page_scan_expression_uses_bounded_viewport_scrolls() {
    let expression = full_page_scan_expression(std::time::Duration::from_millis(250), 7);

    assert!(expression.contains("const delayMs = 250"));
    assert!(expression.contains("const maxSteps = 7"));
    assert!(expression.contains("currentPosition + viewportHeight()"));
    assert!(expression.contains("steps < maxSteps"));
    assert!(expression.contains("window.scrollTo(0, 0)"));
    assert!(expression.contains("window.scrollTo(0, totalHeight)"));
}

#[test]
fn rendered_overlay_cleanup_expression_uses_generic_crawl4ai_rules() {
    let expression = rendered_overlay_cleanup_expression();

    assert!(expression.contains("cookie-banner"));
    assert!(expression.contains("cookie-consent"));
    assert!(expression.contains("newsletter"));
    assert!(expression.contains("role=\"dialog\""));
    assert!(expression.contains("style.position === \"fixed\""));
    assert!(expression.contains("style.position === \"absolute\""));
    assert!(expression.contains("zIndex > 999"));
    assert!(!expression.contains("hellointerview"));
}

#[test]
fn shadow_dom_flatten_expression_resolves_slots_and_skips_styles() {
    let attach_override = shadow_dom_attach_override_expression();
    let flatten = shadow_dom_flatten_expression();

    assert!(attach_override.contains("attachShadow"));
    assert!(attach_override.contains("mode: \"open\""));
    assert!(flatten.contains("shadowRoot"));
    assert!(flatten.contains("assignedNodes({ flatten: true })"));
    assert!(flatten.contains("tag === \"style\""));
    assert!(flatten.contains("serialize(document.documentElement)"));
}
