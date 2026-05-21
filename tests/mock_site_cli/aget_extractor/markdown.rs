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
            "# Reference Bits\n\nStatus: ~~removed~~, _soft_, _under_, `Cmd K`, `TTY`, \"quoted\", HTML.\n\n1\\. Not a generated list.\n\n\\- Not a generated bullet.\n\n\\+ Not a generated plus bullet.\n\nLiteral \\\\*stars\\\\* and \\\\[brackets\\\\].\n\n* * *\n\n> Quoted **block**.\n>\n> Second line.\n\n![Figure alt]({})\n\nFigure caption with source.\n\nExpandable Summary\n\nHidden detail text.\n\nContact the docs team.\n\nTerm\n    Definition with **detail**.\n\n  *[HTML]: HyperText Markup Language",
            site.url("/figure.png")
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
            "# Link Defaults\n\nRead [the guide]({} \"Guide \\\"title\\\" \\[v1\\] \\(draft\\)\") or email support.\n\nCanonical <https://example.com/docs>.\n\nJump [within page]({}).\n\nEmpty []({}) marker.\n\nAsset [release notes]({}) and ![A \\[diagram\\] \\(v1\\)]({}).\n\nIcon [![Download \\[app\\]]({})]({}).",
            site.url("/guide"),
            site.url("/markdown-links#details"),
            site.url("/empty"),
            release_url,
            diagram_url,
            icon_url,
            site.url("/download")
        )
    );
}
