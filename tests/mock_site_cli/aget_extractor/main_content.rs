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
    assert_eq!(main_text.content, "Main Story Useful body text.");

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

    let overlay_text = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/overlay-content"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(overlay_text.content, "Overlay Story Useful article text.");

    let overlay_html = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/overlay-content"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(!overlay_html.content.contains("Cookie banner text."));
    assert!(!overlay_html.content.contains("Newsletter modal text."));
    assert!(overlay_html.content.contains("Useful article text."));

    let main_html = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/main-content"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(main_html.content.contains("<header>Site Header</header>"));
    assert!(main_html.content.contains("<main class=\"story\">"));
}
