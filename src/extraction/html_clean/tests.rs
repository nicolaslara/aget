use scraper::Html;

use super::*;

fn cleaned_html(document: Html) -> String {
    document.root_element().html()
}

#[test]
fn external_link_cleanup_keeps_relative_and_same_base_domain_links() {
    let document = Html::parse_document(
        r#"
<main>
  <a id="relative" href="/guide">Relative</a>
  <a id="same-domain" href="https://www.example.com/docs">Same domain</a>
  <a id="external" href="https://other.test/docs">External</a>
  <a id="mailto" href="mailto:help@example.com">Mail</a>
</main>
"#,
    );

    let cleaned = cleaned_html(
        remove_owned_external_links(document, "https://docs.example.com/page").unwrap(),
    );

    assert!(cleaned.contains("id=\"relative\""));
    assert!(cleaned.contains("id=\"same-domain\""));
    assert!(!cleaned.contains("id=\"external\""));
    assert!(!cleaned.contains("id=\"mailto\""));
}

#[test]
fn internal_link_cleanup_removes_relative_and_same_base_domain_links() {
    let document = Html::parse_document(
        r#"
<main>
  <a id="relative" href="/guide">Relative</a>
  <a id="same-domain" href="https://www.example.com/docs">Same domain</a>
  <a id="external" href="https://other.test/docs">External</a>
  <a id="mailto" href="mailto:help@example.com">Mail</a>
</main>
"#,
    );

    let cleaned = cleaned_html(
        remove_owned_internal_links(document, "https://docs.example.com/page").unwrap(),
    );

    assert!(!cleaned.contains("id=\"relative\""));
    assert!(!cleaned.contains("id=\"same-domain\""));
    assert!(cleaned.contains("id=\"external\""));
    assert!(cleaned.contains("id=\"mailto\""));
}

#[test]
fn external_image_cleanup_keeps_relative_and_same_base_domain_images() {
    let document = Html::parse_document(
        r#"
<main>
  <img id="relative" src="/image.png">
  <img id="same-domain" src="https://assets.example.com/image.png">
  <img id="external" src="https://cdn.other.test/image.png">
  <img id="data" src="data:image/png;base64,AAAA">
</main>
"#,
    );

    let cleaned = cleaned_html(
        remove_owned_external_images(document, "https://docs.example.com/page").unwrap(),
    );

    assert!(cleaned.contains("id=\"relative\""));
    assert!(cleaned.contains("id=\"same-domain\""));
    assert!(!cleaned.contains("id=\"external\""));
    assert!(!cleaned.contains("id=\"data\""));
}

#[test]
fn excluded_domain_cleanup_removes_matching_links_and_images_only() {
    let document = Html::parse_document(
        r#"
<main>
  <a id="relative" href="/guide">Relative</a>
  <a id="blocked-link" href="https://blocked.example/docs">Blocked</a>
  <a id="other-link" href="https://other.test/docs">Other</a>
  <img id="blocked-image" src="https://cdn.blocked.example/image.png">
  <img id="same-domain" src="https://assets.example.com/image.png">
</main>
"#,
    );

    let cleaned = cleaned_html(
        remove_owned_excluded_domain_urls(
            document,
            "https://docs.example.com/page",
            &["blocked.example".to_string()],
        )
        .unwrap(),
    );

    assert!(cleaned.contains("id=\"relative\""));
    assert!(cleaned.contains("id=\"other-link\""));
    assert!(cleaned.contains("id=\"same-domain\""));
    assert!(!cleaned.contains("id=\"blocked-link\""));
    assert!(!cleaned.contains("id=\"blocked-image\""));
}

#[test]
fn social_media_link_cleanup_removes_social_anchors_only() {
    let document = Html::parse_document(
        r#"
<main>
  <a id="guide" href="/guide">Guide</a>
  <a id="social-link" href="https://www.linkedin.com/company/aget">Social</a>
  <a id="custom-social-link" href="https://social.example/company/aget">Custom social</a>
  <img id="social-image" src="https://www.linkedin.com/logo.png">
</main>
"#,
    );

    let cleaned = cleaned_html(
        remove_owned_social_media_links(
            document,
            "https://docs.example.com/page",
            &["social.example".to_string()],
        )
        .unwrap(),
    );

    assert!(cleaned.contains("id=\"guide\""));
    assert!(cleaned.contains("id=\"social-image\""));
    assert!(!cleaned.contains("id=\"social-link\""));
    assert!(!cleaned.contains("id=\"custom-social-link\""));
}
