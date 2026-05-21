use std::path::Path;

use aget::{Aget, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site::MockSite;

pub(super) fn assert_cleanup_outputs(aget_home: &Path, site: &MockSite) {
    let cleaned_html = Aget::new(aget_home)
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
    assert!(!cleaned_html.content.contains("debug-secret"));
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

    let data_attributes = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Html)
        .backend_option("crawl4ai.keep_data_attributes", "true")
        .run()
        .unwrap();
    assert!(data_attributes
        .content
        .contains("data-private=\"main-secret\""));
    assert!(data_attributes.content.contains("data-select=\"summary\""));
    assert!(data_attributes
        .content
        .contains("data-private=\"paragraph-secret\""));
    assert!(data_attributes
        .content
        .contains("data-private=\"link-secret\""));
    assert!(!data_attributes.content.contains("style="));
    assert!(!data_attributes.content.contains("onclick="));
    assert!(!data_attributes.content.contains("aria-label="));
    assert!(!data_attributes.content.contains("rel=\"nofollow\""));

    let selected_by_pruned_attr = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Text)
        .selector(r#"[data-select="summary"]"#)
        .run()
        .unwrap();
    assert_eq!(selected_by_pruned_attr.content, "Visible body.");

    let cleanup_markdown = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert!(!cleanup_markdown.content.contains("data:image"));
    assert!(!cleanup_markdown.content.contains("![Inline image]"));
}
