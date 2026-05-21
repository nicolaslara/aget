use aget::{Aget, AgetExtractorBackend, ErrorCode, OutputFormat};

use crate::aget_extractor_site::aget_extractor_parity_site;
use crate::support::mock_site_cli::save_cookie_session;

#[test]
fn aget_extractor_backend_covers_static_http_parity_slice() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = aget_extractor_parity_site();
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let public = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/public"))
        .content_format(OutputFormat::Text)
        .selector("#content")
        .exclude_selector("nav")
        .run()
        .unwrap();
    assert_eq!(public.extractor, "aget-owned-extractor");
    assert_eq!(public.content, "Public Main Visible public article.");

    let protected = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/protected"))
        .session("app")
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert!(protected.content.contains("Protected Account"));
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));

    let text = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .selector("main.article")
        .exclude_selector("p.ad")
        .run()
        .unwrap();
    assert_eq!(text.content, "Format Heading Format body text.");

    let child_selector = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .selector("main.article > p:not(.ad)")
        .run()
        .unwrap();
    assert_eq!(child_selector.content, "Format body text.");

    let markdown = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    assert_eq!(
        markdown.content,
        format!(
            "# Guide\n\nIntro with **bold** and [docs]({}).\n\nLine one  \nLine two\n\n* First item\n* Second `code`\n\nData Table\n\n| Name | Value |\n| --- | --- |\n| Alpha | [A\\|1]({}) |\n\n```\nlet answer = 42;\n```",
            site.url("/docs"),
            site.url("/alpha")
        )
    );

    let markdown_base = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-base"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    assert_eq!(
        markdown_base.content,
        format!(
            "# Base Links\n\nRead the [base page]({}).",
            site.url("/guide/page.html")
        )
    );

    let markdown_inline_blocks = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-inline-blocks"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    assert_eq!(
        markdown_inline_blocks.content,
        format!(
            "# Reference Bits\n\nStatus: ~~removed~~, _soft_, _under_, `Cmd K`, `TTY`, \"quoted\", HTML.\n\n1\\. Not a generated list.\n\n\\- Not a generated bullet.\n\n\\+ Not a generated plus bullet.\n\nLiteral \\\\*stars\\\\* and \\\\[brackets\\\\].\n\n* * *\n\n> Quoted **block**.\n>\n> Second line.\n\n![Figure alt]({})\n\nFigure caption with source.\n\nExpandable Summary\n\nHidden detail text.\n\nContact the docs team.\n\nTerm\n    Definition with **detail**.\n\n  *[HTML]: HyperText Markup Language",
            site.url("/figure.png")
        )
    );

    let markdown_nested_lists = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-nested-lists"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    assert_eq!(
        markdown_nested_lists.content,
        "# Nested Steps\n\n1. Install\n  * Open settings\n  * Confirm access\n2. Run fetch"
    );

    let markdown_links = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    let release_url = site
        .url("/release(2026)")
        .replace('(', "\\(")
        .replace(')', "\\)");
    let diagram_url = site
        .url("/assets/diagram(1).png")
        .replace('(', "\\(")
        .replace(')', "\\)");
    let icon_url = site
        .url("/icons/app(1).svg")
        .replace('(', "\\(")
        .replace(')', "\\)");
    assert_eq!(
        markdown_links.content,
        format!(
            "# Link Defaults\n\nRead [the guide]({} \"Guide \\\"title\\\" \\[v1\\] \\(draft\\)\") or email support.\n\nCanonical <https://example.com/docs>.\n\nJump within page.\n\nEmpty []({}) marker.\n\nAsset [release notes]({}) and ![A \\[diagram\\] \\(v1\\)]({}).\n\nIcon [![Download \\[app\\]]({})]({}).",
            site.url("/guide"),
            site.url("/empty"),
            release_url,
            diagram_url,
            icon_url,
            site.url("/download")
        )
    );

    let html = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(html.content.contains("<main class=\"article\">"));

    let cleaned_html = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(cleaned_html
        .content
        .contains("<title>Cleanup Title</title>"));
    assert!(cleaned_html.content.contains("<h1>Cleanup Main</h1>"));
    assert!(cleaned_html.content.contains("id=\"cleanup-main\""));
    assert!(cleaned_html.content.contains("class=\"article\""));
    assert!(cleaned_html.content.contains("href=\"/kept\""));
    assert!(cleaned_html.content.contains("title=\"Kept title\""));
    assert!(cleaned_html.content.contains("src=\"/diagram.png\""));
    assert!(cleaned_html.content.contains("alt=\"Diagram\""));
    assert!(cleaned_html.content.contains("width=\"640\""));
    assert!(cleaned_html.content.contains("height=\"480\""));
    assert!(cleaned_html.content.contains("id=\"inline-image\""));
    assert!(cleaned_html.content.contains("alt=\"Inline image\""));
    assert!(cleaned_html.content.contains("id=\"empty-anchor\""));
    assert!(cleaned_html.content.contains("href=\"/empty\""));
    assert!(cleaned_html.content.contains("id=\"empty-break\""));
    assert!(cleaned_html.content.contains("id=\"empty-row\""));
    assert!(cleaned_html.content.contains("id=\"empty-cell\""));
    assert!(cleaned_html.content.contains("id=\"code-space\""));
    assert!(!cleaned_html.content.contains("<meta"));
    assert!(!cleaned_html.content.contains("<link"));
    assert!(!cleaned_html.content.contains("<style"));
    assert!(!cleaned_html.content.contains("<script"));
    assert!(!cleaned_html.content.contains("<noscript"));
    assert!(!cleaned_html.content.contains("data-private"));
    assert!(!cleaned_html.content.contains("data-select"));
    assert!(!cleaned_html.content.contains("style="));
    assert!(!cleaned_html.content.contains("onclick="));
    assert!(!cleaned_html.content.contains("aria-label="));
    assert!(!cleaned_html.content.contains("rel=\"nofollow\""));
    assert!(!cleaned_html.content.contains("data:image/png;base64"));
    assert!(!cleaned_html.content.contains("QUJDRA"));
    assert!(!cleaned_html.content.contains("empty-wrapper"));
    assert!(!cleaned_html.content.contains("empty-span"));

    let selected_by_pruned_attr = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Text)
        .selector(r#"[data-select="summary"]"#)
        .run()
        .unwrap();
    assert_eq!(selected_by_pruned_attr.content, "Visible body.");

    let cleanup_markdown = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert!(!cleanup_markdown.content.contains("data:image"));
    assert!(!cleanup_markdown.content.contains("![Inline image]"));

    let json = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .content_format(OutputFormat::Json)
        .exclude_selector("p.ad")
        .run()
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json.content).unwrap();
    assert_eq!(parsed["url"], site.url("/formats"));
    assert_eq!(parsed["content"], "Format Heading Format body text.");

    let main_text = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(main_text.content, "Main Story Useful body text.");

    let main_markdown = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(main_markdown.content, "# Main Story\n\nUseful body text.");

    let multiple_articles = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/multiple-articles"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(
        multiple_articles.content,
        "# Deep Story\n\nThis article has enough useful body text to beat the promotional card.\n\nIt should be selected as the default main content candidate."
    );

    let labeled_markdown = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/labeled-content"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(
        labeled_markdown.content,
        "# Labeled Story\n\nUseful labeled content should win default extraction without an explicit selector."
    );

    let link_dense_markdown = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content-link-density"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(
        link_dense_markdown.content,
        "# Dense Article\n\nDense useful body text should win because it has direct prose instead of mostly navigation links.\n\nThe local scorer should prefer low-link-density content for agent-ready extraction."
    );

    let overlay_text = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/overlay-content"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(overlay_text.content, "Overlay Story Useful article text.");

    let overlay_html = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/overlay-content"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(!overlay_html.content.contains("Cookie banner text."));
    assert!(!overlay_html.content.contains("Newsletter modal text."));
    assert!(overlay_html.content.contains("Useful article text."));

    let main_html = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(main_html.content.contains("<header>Site Header</header>"));
    assert!(main_html.content.contains("<main class=\"story\">"));

    let selector_miss = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-miss"))
        .content_format(OutputFormat::Text)
        .selector(".does-not-exist")
        .run()
        .unwrap();
    assert_eq!(
        selector_miss.content,
        "Selector Header Selector Main Selector body text. Selector Footer"
    );

    let selector_invalid = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-miss"))
        .content_format(OutputFormat::Text)
        .selector("[[[invalid")
        .run()
        .unwrap();
    assert_eq!(
        selector_invalid.content,
        "Selector Header Selector Main Selector body text. Selector Footer"
    );

    let invalid_exclude_selector = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .selector("main.article")
        .exclude_selector("[[[invalid")
        .run()
        .unwrap();
    assert_eq!(
        invalid_exclude_selector.content,
        "Format Heading Format body text. Promotional aside."
    );

    let selector_multiple = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-multiple"))
        .content_format(OutputFormat::Text)
        .selector(".result")
        .run()
        .unwrap();
    assert_eq!(
        selector_multiple.content,
        "First Result Alpha body. Second Result Beta body."
    );

    let selector_multiple_html = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-multiple"))
        .content_format(OutputFormat::Html)
        .selector(".result")
        .run()
        .unwrap();
    assert!(selector_multiple_html
        .content
        .contains(r#"<section class="result">"#));
    assert!(selector_multiple_html.content.contains("Second Result"));
    assert!(!selector_multiple_html.content.contains("Sidebar body."));

    let selector_scoped_targets = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-multiple"))
        .content_format(OutputFormat::Text)
        .selector(".result")
        .backend_option("crawl4ai.target_elements", "p")
        .run()
        .unwrap();
    assert_eq!(selector_scoped_targets.content, "Alpha body. Beta body.");

    let excluded_tags = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/excluded-tags"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.excluded_tags", "aside,footer")
        .run()
        .unwrap();
    assert_eq!(excluded_tags.content, "Tag Filtering Kept article body.");

    let target_elements = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/excluded-tags"))
        .content_format(OutputFormat::Markdown)
        .backend_option("crawl4ai.target_elements", "h1,p")
        .run()
        .unwrap();
    assert_eq!(
        target_elements.content,
        "# Tag Filtering\n\nKept article body."
    );

    let only_text = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.only_text", "true")
        .run()
        .unwrap();
    assert_eq!(
        only_text.content,
        format!(
            "# Guide\n\nIntro with bold and [docs]({}).\n\nLine one  \nLine two\n\n* First item\n* Second code\n\nData Table\n\n| Name | Value |\n| --- | --- |\n| Alpha | [A\\|1]({}) |\n\n```\nlet answer = 42;\n```",
            site.url("/docs"),
            site.url("/alpha")
        )
    );

    let word_count_threshold = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/word-threshold"))
        .content_format(OutputFormat::Text)
        .selector("main.article")
        .backend_option("crawl4ai.word_count_threshold", "4")
        .run()
        .unwrap();
    assert_eq!(
        word_count_threshold.content,
        "Keep this paragraph because it has enough useful words."
    );

    let invalid_word_count_threshold = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.word_count_threshold", "many")
        .run()
        .unwrap_err();
    assert_eq!(
        invalid_word_count_threshold.code(),
        ErrorCode::ExtractionFailed
    );
    assert!(invalid_word_count_threshold
        .to_string()
        .contains("crawl4ai.word_count_threshold expects a non-negative integer value"));

    let invalid_timeout_option = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.page_timeout", "soon")
        .run()
        .unwrap_err();
    assert_eq!(invalid_timeout_option.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_timeout_option
        .to_string()
        .contains("crawl4ai.page_timeout expects a non-negative integer number of milliseconds"));

    let unsupported_wait_until = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.wait_until", "commit")
        .run()
        .unwrap_err();
    assert_eq!(unsupported_wait_until.code(), ErrorCode::ExtractionFailed);
    assert!(unsupported_wait_until.to_string().contains(
        "crawl4ai.wait_until supports only 'domcontentloaded', 'load', or 'networkidle'"
    ));

    let invalid_wait_for_images = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.wait_for_images", "eventually")
        .run()
        .unwrap_err();
    assert_eq!(invalid_wait_for_images.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_wait_for_images
        .to_string()
        .contains("crawl4ai.wait_for_images expects a boolean value"));

    let unsupported_option = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.magic", "value")
        .run()
        .unwrap_err();
    assert_eq!(unsupported_option.code(), ErrorCode::ExtractionFailed);
    assert!(unsupported_option
        .to_string()
        .contains(
            "supported options: crawl4ai.delay_before_return_html, crawl4ai.excluded_tags, crawl4ai.flatten_shadow_dom, crawl4ai.only_text, crawl4ai.page_timeout, crawl4ai.target_elements, crawl4ai.wait_for_images, crawl4ai.wait_for_timeout, crawl4ai.wait_until, crawl4ai.word_count_threshold"
        ));

    let redirect = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/redirect"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(redirect.final_url, site.url("/public"));

    let waited = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/wait-ready"))
        .content_format(OutputFormat::Text)
        .wait_for_selector("body main #ready")
        .run()
        .unwrap();
    assert_eq!(waited.content, "Ready Now");

    let js_wait = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/wait-ready"))
        .wait_for_selector("js:() => true")
        .run()
        .unwrap_err();
    assert_eq!(js_wait.code(), ErrorCode::ExtractionFailed);
}
