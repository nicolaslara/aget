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
    assert!(cleaned_html.content.contains("id=\"same-domain-link\""));
    assert!(cleaned_html.content.contains("href=\"/same-domain\""));
    assert!(cleaned_html.content.contains("src=\"/diagram.png\""));
    assert!(cleaned_html.content.contains("alt=\"Diagram\""));
    assert!(cleaned_html.content.contains("width=\"640\""));
    assert!(cleaned_html.content.contains("height=\"480\""));
    assert!(cleaned_html.content.contains("id=\"inline-image\""));
    assert!(cleaned_html.content.contains("alt=\"Inline image\""));
    assert!(cleaned_html.content.contains("id=\"empty-anchor\""));
    assert!(cleaned_html.content.contains("href=\"/empty\""));
    assert!(cleaned_html.content.contains("id=\"external-link\""));
    assert!(cleaned_html
        .content
        .contains("href=\"https://external.example/out\""));
    assert!(cleaned_html.content.contains("id=\"mailto-link\""));
    assert!(cleaned_html
        .content
        .contains("href=\"mailto:help@example.com\""));
    assert!(cleaned_html.content.contains("id=\"social-link\""));
    assert!(cleaned_html
        .content
        .contains("href=\"https://www.linkedin.com/company/aget\""));
    assert!(cleaned_html.content.contains("id=\"custom-social-link\""));
    assert!(cleaned_html
        .content
        .contains("href=\"https://social.example/company/aget\""));
    assert!(cleaned_html.content.contains("id=\"remote-image\""));
    assert!(cleaned_html
        .content
        .contains("src=\"https://cdn.example/remote.png\""));
    assert!(cleaned_html.content.contains("id=\"social-image\""));
    assert!(cleaned_html
        .content
        .contains("src=\"https://www.linkedin.com/logo.png\""));
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

    let no_images = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Html)
        .backend_option("crawl4ai.exclude_all_images", "true")
        .run()
        .unwrap();
    assert!(!no_images.content.contains("<img"));
    assert!(!no_images.content.contains("diagram.png"));
    assert!(!no_images.content.contains("Inline image"));

    let no_external_links = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Html)
        .backend_option("crawl4ai.exclude_external_links", "true")
        .run()
        .unwrap();
    assert!(no_external_links.content.contains("href=\"/kept\""));
    assert!(no_external_links
        .content
        .contains("id=\"same-domain-link\""));
    assert!(!no_external_links.content.contains("id=\"external-link\""));
    assert!(!no_external_links
        .content
        .contains("https://external.example/out"));
    assert!(!no_external_links.content.contains("id=\"social-link\""));
    assert!(!no_external_links
        .content
        .contains("id=\"custom-social-link\""));
    assert!(no_external_links.content.contains("id=\"remote-image\""));
    assert!(no_external_links.content.contains("id=\"social-image\""));

    let no_internal_links = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Html)
        .backend_option("crawl4ai.exclude_internal_links", "true")
        .run()
        .unwrap();
    assert!(!no_internal_links.content.contains("id=\"kept-link\""));
    assert!(!no_internal_links
        .content
        .contains("id=\"same-domain-link\""));
    assert!(!no_internal_links.content.contains("href=\"/same-domain\""));
    assert!(!no_internal_links.content.contains("id=\"empty-anchor\""));
    assert!(no_internal_links.content.contains("id=\"external-link\""));
    assert!(no_internal_links
        .content
        .contains("https://external.example/out"));
    assert!(no_internal_links.content.contains("id=\"mailto-link\""));
    assert!(no_internal_links
        .content
        .contains("href=\"mailto:help@example.com\""));
    assert!(no_internal_links.content.contains("id=\"social-link\""));
    assert!(no_internal_links
        .content
        .contains("id=\"custom-social-link\""));
    assert!(no_internal_links.content.contains("id=\"remote-image\""));

    let no_external_images = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Html)
        .backend_option("crawl4ai.exclude_external_images", "true")
        .run()
        .unwrap();
    assert!(no_external_images.content.contains("href=\"/kept\""));
    assert!(no_external_images
        .content
        .contains("href=\"https://external.example/out\""));
    assert!(no_external_images.content.contains("id=\"social-link\""));
    assert!(no_external_images
        .content
        .contains("id=\"custom-social-link\""));
    assert!(no_external_images.content.contains("src=\"/diagram.png\""));
    assert!(!no_external_images.content.contains("id=\"remote-image\""));
    assert!(!no_external_images
        .content
        .contains("https://cdn.example/remote.png"));
    assert!(!no_external_images.content.contains("id=\"social-image\""));

    let no_social_links = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Html)
        .backend_option("crawl4ai.exclude_social_media_links", "true")
        .run()
        .unwrap();
    assert!(no_social_links.content.contains("href=\"/kept\""));
    assert!(no_social_links.content.contains("id=\"external-link\""));
    assert!(no_social_links
        .content
        .contains("id=\"custom-social-link\""));
    assert!(no_social_links.content.contains("id=\"remote-image\""));
    assert!(no_social_links.content.contains("id=\"social-image\""));
    assert!(!no_social_links.content.contains("id=\"social-link\""));
    assert!(!no_social_links
        .content
        .contains("https://www.linkedin.com/company/aget"));

    let no_custom_social_links = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Html)
        .backend_option("crawl4ai.exclude_social_media_links", "true")
        .backend_option("crawl4ai.exclude_social_media_domains", "social.example")
        .run()
        .unwrap();
    assert!(no_custom_social_links.content.contains("href=\"/kept\""));
    assert!(no_custom_social_links
        .content
        .contains("id=\"external-link\""));
    assert!(no_custom_social_links
        .content
        .contains("id=\"remote-image\""));
    assert!(no_custom_social_links
        .content
        .contains("id=\"social-image\""));
    assert!(!no_custom_social_links
        .content
        .contains("id=\"social-link\""));
    assert!(!no_custom_social_links
        .content
        .contains("https://www.linkedin.com/company/aget"));
    assert!(!no_custom_social_links
        .content
        .contains("id=\"custom-social-link\""));
    assert!(!no_custom_social_links
        .content
        .contains("https://social.example/company/aget"));

    let no_excluded_domains = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Html)
        .backend_option("crawl4ai.exclude_domains", "external.example,cdn.example")
        .run()
        .unwrap();
    assert!(no_excluded_domains.content.contains("href=\"/kept\""));
    assert!(no_excluded_domains.content.contains("src=\"/diagram.png\""));
    assert!(!no_excluded_domains.content.contains("id=\"external-link\""));
    assert!(!no_excluded_domains
        .content
        .contains("https://external.example/out"));
    assert!(!no_excluded_domains.content.contains("id=\"remote-image\""));
    assert!(!no_excluded_domains
        .content
        .contains("https://cdn.example/remote.png"));

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
