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
        "Selector Header\nSelector Main\nSelector body text.\nSelector Footer"
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
        "Selector Header\nSelector Main\nSelector body text.\nSelector Footer"
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
        "Format Heading\nFormat body text.\nPromotional aside."
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
        "First Result\nAlpha body.\nSecond Result\nBeta body."
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

    let backend_css_selector = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-multiple"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.css_selector", ".result")
        .run()
        .unwrap();
    assert_eq!(
        backend_css_selector.content,
        "First Result\nAlpha body.\nSecond Result\nBeta body."
    );

    let backend_css_selector_miss = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-miss"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.css_selector", ".does-not-exist")
        .run()
        .unwrap();
    assert_eq!(
        backend_css_selector_miss.content,
        "Selector Header\nSelector Main\nSelector body text.\nSelector Footer"
    );

    let backend_css_selector_invalid = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-miss"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.css_selector", "[[[invalid")
        .run()
        .unwrap();
    assert_eq!(
        backend_css_selector_invalid.content,
        "Selector Header\nSelector Main\nSelector body text.\nSelector Footer"
    );

    let top_level_selector_precedes_backend_css_selector = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-multiple"))
        .content_format(OutputFormat::Text)
        .selector("aside")
        .backend_option("crawl4ai.css_selector", ".result")
        .run()
        .unwrap();
    assert_eq!(
        top_level_selector_precedes_backend_css_selector.content,
        "Sidebar body."
    );

    let selector_scoped_targets = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-multiple"))
        .content_format(OutputFormat::Text)
        .selector(".result")
        .backend_option("crawl4ai.target_elements", "p")
        .run()
        .unwrap();
    assert_eq!(selector_scoped_targets.content, "Alpha body.\nBeta body.");

    let backend_css_selector_scoped_targets = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/selector-multiple"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.css_selector", ".result")
        .backend_option("crawl4ai.target_elements", "p")
        .run()
        .unwrap();
    assert_eq!(
        backend_css_selector_scoped_targets.content,
        "Alpha body.\nBeta body."
    );

    let excluded_tags = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/excluded-tags"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.excluded_tags", "aside,footer")
        .run()
        .unwrap();
    assert_eq!(excluded_tags.content, "Tag Filtering\nKept article body.");

    let excluded_selector = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/excluded-tags"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.excluded_selector", "aside,footer")
        .run()
        .unwrap();
    assert_eq!(
        excluded_selector.content,
        "Tag Filtering\nKept article body."
    );

    let invalid_excluded_selector = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/excluded-tags"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.excluded_selector", "[[[invalid")
        .run()
        .unwrap();
    assert_eq!(
        invalid_excluded_selector.content,
        "Tag Filtering\nPromotional Sidebar\nKept article body.\nArticle Footer"
    );

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

    let targeted_only_text_root = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown"))
        .content_format(OutputFormat::Html)
        .backend_option("crawl4ai.target_elements", "strong")
        .backend_option("crawl4ai.only_text", "true")
        .run()
        .unwrap();
    assert_eq!(targeted_only_text_root.content, "<strong>bold</strong>");

    let forms_by_default = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/remove-forms"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(
        forms_by_default.content,
        "Form Cleanup\nPrivate form label\nKept content."
    );

    let removed_forms = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/remove-forms"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.remove_forms", "true")
        .run()
        .unwrap();
    assert_eq!(removed_forms.content, "Form Cleanup\nKept content.");
}
