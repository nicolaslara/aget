use std::{fs, path::Path};

use aget::{Aget, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site::MockSite;

pub(super) fn assert_public_session_and_formats(aget_home: &Path, site: &MockSite) {
    let public = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/public"))
        .content_format(OutputFormat::Text)
        .selector("#content")
        .exclude_selector("nav")
        .run()
        .unwrap();
    assert_eq!(public.extractor, "aget-owned-extractor");
    assert_eq!(public.content, "Public Main\nVisible public article.");

    let protected = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/protected"))
        .session("app")
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert!(protected.content.contains("Protected Account"));
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));

    let text = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .selector("main.article")
        .exclude_selector("p.ad")
        .run()
        .unwrap();
    assert_eq!(text.content, "Format Heading\nFormat body text.");

    let child_selector = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .selector("main.article > p:not(.ad)")
        .run()
        .unwrap();
    assert_eq!(child_selector.content, "Format body text.");

    let html = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(html.content.contains("<main class=\"article\">"));

    let json = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .content_format(OutputFormat::Json)
        .exclude_selector("p.ad")
        .run()
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json.content).unwrap();
    assert_eq!(parsed["url"], site.url("/formats"));
    assert_eq!(parsed["content"], "Format Heading\nFormat body text.");

    let metadata = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/metadata"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(metadata.content, "Metadata Body\nMetadata body text.");
    assert_eq!(metadata.page_metadata["title"], "Metadata Title");
    assert_eq!(
        metadata.page_metadata["description"],
        "Metadata description"
    );
    assert_eq!(metadata.page_metadata["keywords"], "agent,context,local");
    assert_eq!(metadata.page_metadata["author"], "Aget Docs");
    assert_eq!(metadata.page_metadata["og:title"], "Open Graph Title");
    assert_eq!(
        metadata.page_metadata["og:description"],
        "Open Graph description"
    );
    assert_eq!(metadata.page_metadata["twitter:title"], "Twitter Title");
    assert_eq!(
        metadata.page_metadata["article:published_time"],
        "2026-05-22T10:00:00Z"
    );
    let artifact_metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&metadata.artifacts.metadata).unwrap()).unwrap();
    assert_eq!(
        artifact_metadata["page_metadata"]["description"],
        "Metadata description"
    );

    let fallback_metadata = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/metadata-title-fallback"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(
        fallback_metadata.content,
        "Fallback Metadata Body\nFallback metadata body text."
    );
    assert_eq!(
        fallback_metadata.page_metadata["title"],
        "Fallback Open Graph Title"
    );
    assert_eq!(
        fallback_metadata.page_metadata["og:title"],
        "Fallback Open Graph Title"
    );
    assert_eq!(
        fallback_metadata.page_metadata["twitter:title"],
        "Fallback Twitter Title"
    );
}
