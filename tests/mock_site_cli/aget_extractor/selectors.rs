use std::path::Path;

use aget::{Aget, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site::MockSite;

pub(super) fn assert_selector_and_target_options(aget_home: &Path, site: &MockSite) {
    let selector_miss = Aget::new(aget_home)
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

    let selector_invalid = Aget::new(aget_home)
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

    let invalid_exclude_selector = Aget::new(aget_home)
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

    let selector_multiple = Aget::new(aget_home)
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

    let selector_multiple_html = Aget::new(aget_home)
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

    let selector_scoped_targets = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-multiple"))
        .content_format(OutputFormat::Text)
        .selector(".result")
        .backend_option("crawl4ai.target_elements", "p")
        .run()
        .unwrap();
    assert_eq!(selector_scoped_targets.content, "Alpha body. Beta body.");

    let excluded_tags = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/excluded-tags"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.excluded_tags", "aside,footer")
        .run()
        .unwrap();
    assert_eq!(excluded_tags.content, "Tag Filtering Kept article body.");

    let target_elements = Aget::new(aget_home)
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
}
