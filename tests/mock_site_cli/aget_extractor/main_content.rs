use std::path::Path;

use aget::{Aget, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site::MockSite;

pub(super) fn assert_main_content_scoring(aget_home: &Path, site: &MockSite) {
    let main_text = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(main_text.content, "Main Story\nUseful body text.");

    let main_markdown = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(main_markdown.content, "# Main Story\n\nUseful body text.");

    let multiple_articles = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/multiple-articles"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(
        multiple_articles.content,
        "# Deep Story\n\nThis article has enough useful body text to beat the promotional card.\n\nIt should be selected as the default main content candidate."
    );

    let labeled_markdown = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/labeled-content"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(
        labeled_markdown.content,
        "# Labeled Story\n\nUseful labeled content should win default extraction without an explicit selector."
    );

    let page_chrome_markdown = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content-page-chrome"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(
        page_chrome_markdown.content,
        "# Real Article\n\nThe real article should win because Crawl4AI-style pruning ignores candidates inside page chrome."
    );

    let link_dense_markdown = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content-link-density"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(
        link_dense_markdown.content,
        "# Dense Article\n\nDense useful body text should win because it has direct prose instead of mostly navigation links.\n\nThe local scorer should prefer low-link-density content for agent-ready extraction."
    );

    let unlabeled_density_markdown = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content-unlabeled-density"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(
        unlabeled_density_markdown.content,
        "# Unlabeled Report\n\nThis unlabeled report carries the main body text even though it has no helpful class, id, role, or aria label.\n\nThe content block should win because it has strong prose density and very few links."
    );

    let body_fallback_markdown = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content-body-fallback-chrome"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(
        body_fallback_markdown.content,
        "# Bare Body Story\n\nThe useful article paragraph has no wrapper, so body fallback should keep it without page chrome."
    );

    let class_id_noise_markdown = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content-class-id-noise"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(
        class_id_noise_markdown.content,
        "# Primary Article\n\nThe primary article should win even when a noisy comments block has enough text to look important."
    );

    let embedded_noise_label_markdown = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content-embedded-noise-label"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(
        embedded_noise_label_markdown.content,
        "# Source Article\n\nThe source article should win because embedded comments labels exclude noisy candidates."
    );

    let threshold_markdown = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content-word-threshold"))
        .content_format(OutputFormat::Markdown)
        .backend_option("crawl4ai.word_count_threshold", "9")
        .run()
        .unwrap();
    assert_eq!(
        threshold_markdown.content,
        "# Long Article\n\nThis fallback article has enough useful words to pass the configured threshold."
    );
    assert!(!threshold_markdown.content.contains("Short Teaser"));

    let overlay_text = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/overlay-content"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(overlay_text.content, "Overlay Story\nUseful article text.");

    let overlay_html = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/overlay-content"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(!overlay_html.content.contains("Cookie banner text."));
    assert!(!overlay_html.content.contains("Newsletter modal text."));
    assert!(overlay_html.content.contains("Useful article text."));

    let overlay_html_without_cleanup = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/overlay-content"))
        .content_format(OutputFormat::Html)
        .backend_option("crawl4ai.remove_overlay_elements", "false")
        .run()
        .unwrap();
    assert!(overlay_html_without_cleanup
        .content
        .contains("Cookie banner text."));
    assert!(overlay_html_without_cleanup
        .content
        .contains("Newsletter modal text."));
    assert!(overlay_html_without_cleanup
        .content
        .contains("Useful article text."));

    let main_html = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(main_html.content.contains("<header>Site Header</header>"));
    assert!(main_html.content.contains("<main class=\"story\">"));
}
