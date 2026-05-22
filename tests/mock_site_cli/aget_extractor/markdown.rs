use std::path::Path;

use aget::{Aget, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site::MockSite;

pub(super) fn assert_markdown_rendering(aget_home: &Path, site: &MockSite) {
    let markdown = Aget::new(aget_home)
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

    let markdown_ignore_tables = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.ignore_tables", "true")
        .run()
        .unwrap();
    assert_eq!(
        markdown_ignore_tables.content,
        format!(
            "# Guide\n\nIntro with **bold** and [docs]({}).\n\nLine one  \nLine two\n\n* First item\n* Second `code`\n\nData Table\nName Value\nAlpha [A|1]({})\n\n```\nlet answer = 42;\n```",
            site.url("/docs"),
            site.url("/alpha")
        )
    );

    let markdown_bypass_tables = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.bypass_tables", "true")
        .run()
        .unwrap();
    assert_eq!(
        markdown_bypass_tables.content,
        format!(
            "# Guide\n\nIntro with **bold** and [docs]({}).\n\nLine one  \nLine two\n\n* First item\n* Second `code`\n\n<table>Data Table\n<tr><th>Name</th><th>Value</th></tr><tr><td>Alpha</td><td>[A|1]({})</td></tr></table>\n\n```\nlet answer = 42;\n```",
            site.url("/docs"),
            site.url("/alpha")
        )
    );

    let markdown_base = Aget::new(aget_home)
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

    let markdown_inline_blocks = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-inline-blocks"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    assert_eq!(
        markdown_inline_blocks.content,
        format!(
            "# Reference Bits\n\nStatus: ~~removed~~, _soft_, _under_, `Cmd K`, `TTY`, \"quoted\", HTML, power 2, and water 2.\n\n1\\. Not a generated list.\n\n\\- Not a generated bullet.\n\n\\+ Not a generated plus bullet.\n\nLiteral \\\\*stars\\\\* and \\\\[brackets\\\\].\n\n* * *\n\n> Quoted **block**.\n>\n> Second line.\n\n![Figure alt]({})\n\nFigure caption with source.\n\nExpandable Summary\n\nHidden detail text.\n\nContact the docs team.\n\nTerm\n    Definition with **detail**.\n\n  *[HTML]: HyperText Markup Language",
            site.url("/figure.png")
        )
    );

    let markdown_include_sup_sub = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-inline-blocks"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.include_sup_sub", "true")
        .run()
        .unwrap();
    assert!(markdown_include_sup_sub
        .content
        .contains("power <sup>2</sup>, and water <sub>2</sub>."));

    let markdown_ignore_emphasis = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-inline-blocks"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.ignore_emphasis", "true")
        .run()
        .unwrap();
    assert!(markdown_ignore_emphasis
        .content
        .contains("Status: ~~removed~~, soft, under, `Cmd K`, `TTY`, \"quoted\""));
    assert!(markdown_ignore_emphasis
        .content
        .contains("Definition with detail."));
    assert!(!markdown_ignore_emphasis.content.contains("_soft_"));
    assert!(!markdown_ignore_emphasis.content.contains("_under_"));
    assert!(!markdown_ignore_emphasis.content.contains("**detail**"));
    assert!(markdown_ignore_emphasis.content.contains("~~removed~~"));

    let markdown_emphasis_markers = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-inline-blocks"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.emphasis_mark", "*")
        .backend_option("crawl4ai.strong_mark", "__")
        .run()
        .unwrap();
    assert!(markdown_emphasis_markers
        .content
        .contains("Status: ~~removed~~, *soft*, *under*,"));
    assert!(markdown_emphasis_markers
        .content
        .contains("Definition with __detail__."));
    assert!(!markdown_emphasis_markers.content.contains("_soft_"));
    assert!(!markdown_emphasis_markers.content.contains("**detail**"));

    let markdown_quote_markers = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-inline-blocks"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.open_quote", "<<")
        .backend_option("crawl4ai.close_quote", ">>")
        .run()
        .unwrap();
    assert!(markdown_quote_markers
        .content
        .contains("`Cmd K`, `TTY`, <<quoted>>, HTML"));

    let markdown_escape_snob = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-escape-snob"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.escape_snob", "true")
        .run()
        .unwrap();
    assert_eq!(
        markdown_escape_snob.content,
        concat!(
            "# Escape Snob\n\n",
            "Escapes \\`tick\\`, \\*star\\*, \\_under\\_, \\{brace\\}, ",
            "\\[bracket\\], \\(paren\\), \\#hash, and bang\\!.\n\n",
            "`*code*` stays code.\n\n",
            "```\n",
            "# raw *code*\n",
            "```"
        )
    );

    let markdown_nested_lists = Aget::new(aget_home)
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

    let markdown_unordered_marker = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-nested-lists"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.ul_item_mark", "-")
        .run()
        .unwrap();
    assert_eq!(
        markdown_unordered_marker.content,
        "# Nested Steps\n\n1. Install\n  - Open settings\n  - Confirm access\n2. Run fetch"
    );

    let markdown_code_whitespace = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-code-whitespace"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    assert_eq!(
        markdown_code_whitespace.content,
        concat!(
            "# Code Whitespace\n\n",
            "```\n",
            "first\n",
            "let padded = true;  \n",
            "\n",
            "last\n",
            "```"
        )
    );

    let markdown_links = Aget::new(aget_home)
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
            "# Link Defaults\n\n## [Linked Heading]({} \"Heading title\")\n\nRead [the guide]({} \"Guide \\\"title\\\" \\[v1\\] \\(draft\\)\") or email support.\n\nCanonical <https://example.com/docs>.\n\nJump [within page]({}).\n\nLink label [API v1]({}) and standalone `inline_code`.\n\nEmpty []({}) marker.\n\nAsset [release notes]({}) and ![A \\[diagram\\] \\(v1\\)]({}).\n\nIcon [![Download \\[app\\]]({})]({}).\n\nMissing ![]({}) and empty ![]({}).",
            site.url("/linked-heading"),
            site.url("/guide"),
            site.url("/markdown-links#details"),
            site.url("/api"),
            site.url("/empty"),
            release_url,
            diagram_url,
            icon_url,
            site.url("/download"),
            site.url("/missing-alt.png"),
            site.url("/empty-alt.png")
        )
    );

    let markdown_disable_automatic_links = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.use_automatic_links", "false")
        .run()
        .unwrap();
    assert!(markdown_disable_automatic_links.content.contains(
        "Canonical [https://example.com/docs](https://example.com/docs \"Docs title\")."
    ));
    assert!(!markdown_disable_automatic_links
        .content
        .contains("Canonical <https://example.com/docs>."));

    let markdown_ignore_images = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.ignore_images", "true")
        .run()
        .unwrap();
    assert!(markdown_ignore_images
        .content
        .contains("Asset [release notes]("));
    assert!(markdown_ignore_images
        .content
        .contains(&format!("Icon []({}).", site.url("/download"))));
    assert!(!markdown_ignore_images.content.contains("![A \\[diagram\\]"));
    assert!(!markdown_ignore_images.content.contains("/assets/diagram"));
    assert!(!markdown_ignore_images
        .content
        .contains("![Download \\[app\\]]"));

    let markdown_images_to_alt = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.images_to_alt", "true")
        .run()
        .unwrap();
    assert!(markdown_images_to_alt
        .content
        .contains("Canonical <https://example.com/docs>."));
    assert!(markdown_images_to_alt.content.contains(&format!(
        "Asset [release notes]({}) and A \\[diagram\\] \\(v1\\).",
        release_url
    )));
    assert!(markdown_images_to_alt.content.contains(&format!(
        "Icon [Download \\[app\\]]({}).",
        site.url("/download")
    )));
    assert!(!markdown_images_to_alt.content.contains("![A \\[diagram\\]"));
    assert!(!markdown_images_to_alt.content.contains("/assets/diagram"));
    assert!(!markdown_images_to_alt
        .content
        .contains("![Download \\[app\\]]"));

    let markdown_default_image_alt = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.default_image_alt", "Missing alt")
        .run()
        .unwrap();
    assert!(markdown_default_image_alt.content.contains(&format!(
        "Missing ![Missing alt]({}) and empty ![Missing alt]({}).",
        site.url("/missing-alt.png"),
        site.url("/empty-alt.png")
    )));

    let markdown_default_alt_to_alt = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.default_image_alt", "Missing alt")
        .backend_option("crawl4ai.images_to_alt", "true")
        .run()
        .unwrap();
    assert!(markdown_default_alt_to_alt
        .content
        .contains("Missing Missing alt and empty Missing alt."));

    let markdown_images_as_html = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.images_as_html", "true")
        .backend_option("crawl4ai.images_to_alt", "true")
        .run()
        .unwrap();
    assert!(markdown_images_as_html
        .content
        .contains("Asset [release notes]("));
    assert!(markdown_images_as_html.content.contains(
        "and <img src='/assets/diagram(1).png' width='640' height='360' alt='A [diagram] (v1)' />."
    ));
    assert!(markdown_images_as_html.content.contains(&format!(
        "Icon [<img src='/icons/app(1).svg' width='32' alt='Download [app]' />]({}).",
        site.url("/download")
    )));
    assert!(!markdown_images_as_html
        .content
        .contains("![A \\[diagram\\]"));
    assert!(!markdown_images_as_html
        .content
        .contains("A \\[diagram\\] \\(v1\\)."));

    let markdown_default_alt_as_html = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.default_image_alt", "Missing alt")
        .backend_option("crawl4ai.images_as_html", "true")
        .run()
        .unwrap();
    assert!(markdown_default_alt_as_html.content.contains(
        "Missing <img src='/missing-alt.png' alt='Missing alt' /> and empty <img src='/empty-alt.png' alt='Missing alt' />."
    ));

    let markdown_images_with_size = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.images_with_size", "true")
        .backend_option("crawl4ai.images_to_alt", "true")
        .run()
        .unwrap();
    assert!(markdown_images_with_size.content.contains(
        "and <img src='/assets/diagram(1).png' width='640' height='360' alt='A [diagram] (v1)' />."
    ));
    assert!(markdown_images_with_size.content.contains(&format!(
        "Icon [<img src='/icons/app(1).svg' width='32' alt='Download [app]' />]({}).",
        site.url("/download")
    )));
    assert!(!markdown_images_with_size
        .content
        .contains("![A \\[diagram\\]"));
    assert!(!markdown_images_with_size
        .content
        .contains("A \\[diagram\\] \\(v1\\)."));

    let markdown_images_with_size_keeps_unsized_images = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-inline-blocks"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.images_with_size", "true")
        .run()
        .unwrap();
    assert!(markdown_images_with_size_keeps_unsized_images
        .content
        .contains(&format!("![Figure alt]({})", site.url("/figure.png"))));
    assert!(!markdown_images_with_size_keeps_unsized_images
        .content
        .contains("<img src='/figure.png'"));

    let markdown_ignore_links = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.ignore_links", "true")
        .run()
        .unwrap();
    assert!(markdown_ignore_links.content.contains("## Linked Heading"));
    assert!(markdown_ignore_links
        .content
        .contains("Read the guide or email support."));
    assert!(markdown_ignore_links
        .content
        .contains("Icon ![Download \\[app\\]]("));
    assert!(!markdown_ignore_links.content.contains("[the guide]("));
    assert!(!markdown_ignore_links.content.contains("[Linked Heading]("));
    assert!(!markdown_ignore_links
        .content
        .contains("Guide \\\"title\\\""));
    assert!(!markdown_ignore_links
        .content
        .contains(&site.url("/download")));

    let markdown_include_mailto_links = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.ignore_mailto_links", "false")
        .run()
        .unwrap();
    assert!(markdown_include_mailto_links
        .content
        .contains("Read [the guide]("));
    assert!(markdown_include_mailto_links
        .content
        .contains("[email support](mailto:help@example.com)."));

    let markdown_protect_links = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.protect_links", "true")
        .run()
        .unwrap();
    assert!(markdown_protect_links.content.contains(&format!(
        "## [Linked Heading](<{}> \"Heading title\")",
        site.url("/linked-heading")
    )));
    assert!(markdown_protect_links.content.contains(&format!(
        "[the guide](<{}> \"Guide \\\"title\\\" \\[v1\\] \\(draft\\)\")",
        site.url("/guide")
    )));
    assert!(markdown_protect_links.content.contains(&format!(
        "[release notes](<{}>)",
        site.url("/release(2026)")
    )));
    assert!(markdown_protect_links
        .content
        .contains("<https://example.com/docs>"));
    assert!(markdown_protect_links.content.contains(&format!(
        "[![Download \\[app\\]]({})](<{}>)",
        icon_url,
        site.url("/download")
    )));

    let markdown_skip_internal_links = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.skip_internal_links", "true")
        .run()
        .unwrap();
    assert!(markdown_skip_internal_links
        .content
        .contains("Jump within page."));
    assert!(!markdown_skip_internal_links
        .content
        .contains("/markdown-links#details"));

    let markdown_ordered_start = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/markdown-ordered-start"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    assert_eq!(
        markdown_ordered_start.content,
        "# Ordered Start\n\n1. Resume\n2. Verify\n\n1. Fallback"
    );
}
