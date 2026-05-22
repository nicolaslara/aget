use std::path::Path;

use aget::{Aget, AgetExtractorBackend, ErrorCode, OutputFormat};

use crate::support::mock_site::MockSite;

pub(super) fn assert_content_options(aget_home: &Path, site: &MockSite) {
    let only_text = Aget::new(aget_home)
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

    let only_text_html = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown"))
        .content_format(OutputFormat::Html)
        .selector("main.article")
        .backend_option("crawl4ai.only_text", "true")
        .run()
        .unwrap();
    assert!(only_text_html.content.contains("Intro with bold and"));
    assert!(only_text_html.content.contains("Second code"));
    assert!(!only_text_html.content.contains("<strong>"));
    assert!(!only_text_html.content.contains("<span>"));
    assert!(!only_text_html.content.contains("<code>code</code>"));
    assert!(only_text_html
        .content
        .contains("<a href=\"/docs\">docs</a>"));

    let word_count_threshold = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/word-threshold"))
        .content_format(OutputFormat::Text)
        .selector("main.article")
        .backend_option("crawl4ai.word_count_threshold", "4")
        .run()
        .unwrap();
    assert_eq!(
        word_count_threshold.content,
        "Threshold Example Title\nKeep this paragraph because it has enough useful words.\nTiny caption\nShort link"
    );

    let threshold_html = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/word-threshold"))
        .content_format(OutputFormat::Html)
        .selector("main.article")
        .backend_option("crawl4ai.word_count_threshold", "9")
        .run()
        .unwrap();
    assert!(threshold_html.content.contains("Tiny caption"));
    assert!(threshold_html.content.contains("Short link"));
    assert!(threshold_html
        .content
        .contains("<pre><code><span> </span></code></pre>"));

    let cache_bypass = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .selector("main.article")
        .backend_option("crawl4ai.cache", "bypass")
        .backend_option("crawl4ai.cache_mode", "disabled")
        .run()
        .unwrap();
    assert_eq!(
        cache_bypass.content,
        "Format Heading\nFormat body text.\nPromotional aside."
    );

    let cache_enabled = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.cache_mode", "enabled")
        .run()
        .unwrap_err();
    assert_eq!(cache_enabled.code(), ErrorCode::ExtractionFailed);
    assert!(cache_enabled
        .to_string()
        .contains("requires an owned cache store"));

    let user_agent = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .selector("main.article")
        .backend_option("crawl4ai.user_agent", "aget-test/2.0")
        .run()
        .unwrap();
    assert_eq!(
        user_agent.content,
        "Format Heading\nFormat body text.\nPromotional aside."
    );
    assert!(site.received_header("/formats", "user-agent", "aget-test/2.0"));
}
