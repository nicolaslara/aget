use std::path::Path;

use aget::{Aget, AgetExtractorBackend, ErrorCode, OutputFormat};

use crate::support::mock_site::MockSite;

pub(super) fn assert_backend_options_redirects_and_waits(aget_home: &Path, site: &MockSite) {
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

    let invalid_word_count_threshold = Aget::new(aget_home)
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

    let invalid_body_width = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.body_width", "wide")
        .run()
        .unwrap_err();
    assert_eq!(invalid_body_width.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_body_width
        .to_string()
        .contains("crawl4ai.body_width expects a non-negative integer value"));

    let invalid_mark_code = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.mark_code", "maybe")
        .run()
        .unwrap_err();
    assert_eq!(invalid_mark_code.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_mark_code
        .to_string()
        .contains("crawl4ai.mark_code expects a boolean value"));

    let invalid_single_line_break = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.single_line_break", "sometimes")
        .run()
        .unwrap_err();
    assert_eq!(
        invalid_single_line_break.code(),
        ErrorCode::ExtractionFailed
    );
    assert!(invalid_single_line_break
        .to_string()
        .contains("crawl4ai.single_line_break expects a boolean value"));

    let invalid_unicode_snob = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.unicode_snob", "maybe")
        .run()
        .unwrap_err();
    assert_eq!(invalid_unicode_snob.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_unicode_snob
        .to_string()
        .contains("crawl4ai.unicode_snob expects a boolean value"));

    let invalid_wrap_links = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.wrap_links", "maybe")
        .run()
        .unwrap_err();
    assert_eq!(invalid_wrap_links.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_wrap_links
        .to_string()
        .contains("crawl4ai.wrap_links expects a boolean value"));

    let invalid_inline_links = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.inline_links", "maybe")
        .run()
        .unwrap_err();
    assert_eq!(invalid_inline_links.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_inline_links
        .to_string()
        .contains("crawl4ai.inline_links expects a boolean value"));

    let invalid_links_each_paragraph = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.links_each_paragraph", "maybe")
        .run()
        .unwrap_err();
    assert_eq!(
        invalid_links_each_paragraph.code(),
        ErrorCode::ExtractionFailed
    );
    assert!(invalid_links_each_paragraph
        .to_string()
        .contains("crawl4ai.links_each_paragraph expects a boolean value"));

    let invalid_wrap_list_items = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.wrap_list_items", "maybe")
        .run()
        .unwrap_err();
    assert_eq!(invalid_wrap_list_items.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_wrap_list_items
        .to_string()
        .contains("crawl4ai.wrap_list_items expects a boolean value"));

    let invalid_wrap_tables = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.wrap_tables", "maybe")
        .run()
        .unwrap_err();
    assert_eq!(invalid_wrap_tables.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_wrap_tables
        .to_string()
        .contains("crawl4ai.wrap_tables expects a boolean value"));

    let invalid_pad_tables = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.pad_tables", "maybe")
        .run()
        .unwrap_err();
    assert_eq!(invalid_pad_tables.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_pad_tables
        .to_string()
        .contains("crawl4ai.pad_tables expects a boolean value"));

    let invalid_hide_strikethrough = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.hide_strikethrough", "maybe")
        .run()
        .unwrap_err();
    assert_eq!(
        invalid_hide_strikethrough.code(),
        ErrorCode::ExtractionFailed
    );
    assert!(invalid_hide_strikethrough
        .to_string()
        .contains("crawl4ai.hide_strikethrough expects a boolean value"));

    let invalid_timeout_option = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.page_timeout", "soon")
        .run()
        .unwrap_err();
    assert_eq!(invalid_timeout_option.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_timeout_option
        .to_string()
        .contains("crawl4ai.page_timeout expects a non-negative integer number of milliseconds"));

    let unsupported_wait_until = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.wait_until", "commit")
        .run()
        .unwrap_err();
    assert_eq!(unsupported_wait_until.code(), ErrorCode::ExtractionFailed);
    assert!(unsupported_wait_until.to_string().contains(
        "crawl4ai.wait_until supports only 'domcontentloaded', 'load', or 'networkidle'"
    ));

    let invalid_base_url = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.base_url", "/relative/")
        .run()
        .unwrap_err();
    assert_eq!(invalid_base_url.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_base_url
        .to_string()
        .contains("crawl4ai.base_url expects an absolute URL"));

    let invalid_wait_for_images = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.wait_for_images", "eventually")
        .run()
        .unwrap_err();
    assert_eq!(invalid_wait_for_images.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_wait_for_images
        .to_string()
        .contains("crawl4ai.wait_for_images expects a boolean value"));

    let invalid_process_iframes = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.process_iframes", "maybe")
        .run()
        .unwrap_err();
    assert_eq!(invalid_process_iframes.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_process_iframes
        .to_string()
        .contains("crawl4ai.process_iframes expects a boolean value"));

    let invalid_scroll_delay = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.scroll_delay", "later")
        .run()
        .unwrap_err();
    assert_eq!(invalid_scroll_delay.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_scroll_delay
        .to_string()
        .contains("crawl4ai.scroll_delay expects a non-negative number of seconds"));

    let invalid_max_scroll_steps = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.max_scroll_steps", "many")
        .run()
        .unwrap_err();
    assert_eq!(invalid_max_scroll_steps.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_max_scroll_steps
        .to_string()
        .contains("crawl4ai.max_scroll_steps expects a non-negative integer value"));

    let unsupported_option = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/formats"))
        .backend_option("crawl4ai.magic", "value")
        .run()
        .unwrap_err();
    assert_eq!(unsupported_option.code(), ErrorCode::ExtractionFailed);
    assert!(unsupported_option
        .to_string()
        .contains(
            "supported options: crawl4ai.base_url, crawl4ai.body_width, crawl4ai.bypass_tables, crawl4ai.close_quote, crawl4ai.default_image_alt, crawl4ai.delay_before_return_html, crawl4ai.emphasis_mark, crawl4ai.escape_backslash, crawl4ai.escape_dash, crawl4ai.escape_dot, crawl4ai.escape_plus, crawl4ai.escape_snob, crawl4ai.exclude_all_images, crawl4ai.exclude_domains, crawl4ai.exclude_external_images, crawl4ai.exclude_external_links, crawl4ai.exclude_internal_links, crawl4ai.exclude_social_media_domains, crawl4ai.exclude_social_media_links, crawl4ai.excluded_tags, crawl4ai.flatten_shadow_dom, crawl4ai.hide_strikethrough, crawl4ai.ignore_emphasis, crawl4ai.ignore_images, crawl4ai.ignore_links, crawl4ai.ignore_mailto_links, crawl4ai.ignore_tables, crawl4ai.images_as_html, crawl4ai.images_to_alt, crawl4ai.images_with_size, crawl4ai.include_sup_sub, crawl4ai.inline_links, crawl4ai.keep_data_attributes, crawl4ai.links_each_paragraph, crawl4ai.mark_code, crawl4ai.max_scroll_steps, crawl4ai.only_text, crawl4ai.open_quote, crawl4ai.pad_tables, crawl4ai.page_timeout, crawl4ai.process_iframes, crawl4ai.protect_links, crawl4ai.remove_forms, crawl4ai.remove_overlay_elements, crawl4ai.scan_full_page, crawl4ai.scroll_delay, crawl4ai.single_line_break, crawl4ai.skip_internal_links, crawl4ai.strong_mark, crawl4ai.target_elements, crawl4ai.ul_item_mark, crawl4ai.unicode_snob, crawl4ai.use_automatic_links, crawl4ai.wait_for_images, crawl4ai.wait_for_timeout, crawl4ai.wait_until, crawl4ai.word_count_threshold, crawl4ai.wrap_links, crawl4ai.wrap_list_items, crawl4ai.wrap_tables"
        ));

    let redirect = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/redirect"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(redirect.final_url, site.url("/public"));

    let waited = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/wait-ready"))
        .content_format(OutputFormat::Text)
        .wait_for_selector("css:body main #ready")
        .run()
        .unwrap();
    assert_eq!(waited.content, "Ready Now");

    let js_wait = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/wait-ready"))
        .wait_for_selector("js:() => true")
        .run()
        .unwrap_err();
    assert_eq!(js_wait.code(), ErrorCode::ExtractionFailed);
}
