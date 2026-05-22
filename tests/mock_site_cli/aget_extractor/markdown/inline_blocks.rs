use std::path::Path;

use crate::support::mock_site::MockSite;

use super::markdown_content;

pub(super) fn assert_inline_blocks(aget_home: &Path, site: &MockSite) {
    let markdown_inline_blocks = markdown_content(aget_home, site, "/markdown-inline-blocks", &[]);
    assert_eq!(
        markdown_inline_blocks,
        format!(
            "# Reference Bits\n\nStatus: ~~removed~~, _soft_, _under_, `Cmd K`, `TTY`, \"quoted\", HTML, power 2, and water 2.\n\nStyled removed text.\n\n1\\. Not a generated list.\n\n\\- Not a generated bullet.\n\n\\+ Not a generated plus bullet.\n\nLiteral \\\\*stars\\\\* and \\\\[brackets\\\\].\n\n* * *\n\n> Quoted **block**.\n>\n> Second line.\n\n![Figure alt]({})\n\nFigure caption with source.\n\nExpandable Summary\n\nHidden detail text.\n\nContact the docs team.\n\nTerm\n    Definition with **detail**.\n\n  *[HTML]: HyperText Markup Language",
            site.url("/figure.png")
        )
    );

    let markdown_line_start_escapes = markdown_content(
        aget_home,
        site,
        "/markdown-inline-blocks",
        &[
            ("crawl4ai.escape_dot", "false"),
            ("crawl4ai.escape_dash", "false"),
            ("crawl4ai.escape_plus", "false"),
        ],
    );
    assert!(markdown_line_start_escapes.contains("\n\n1. Not a generated list.\n\n"));
    assert!(markdown_line_start_escapes.contains("\n\n- Not a generated bullet.\n\n"));
    assert!(markdown_line_start_escapes.contains("\n\n+ Not a generated plus bullet.\n\n"));
    assert!(!markdown_line_start_escapes.contains("1\\. Not a generated list."));
    assert!(!markdown_line_start_escapes.contains("\\- Not a generated bullet."));
    assert!(!markdown_line_start_escapes.contains("\\+ Not a generated plus bullet."));

    let markdown_include_sup_sub = markdown_content(
        aget_home,
        site,
        "/markdown-inline-blocks",
        &[("crawl4ai.include_sup_sub", "true")],
    );
    assert!(markdown_include_sup_sub.contains("power <sup>2</sup>, and water <sub>2</sub>."));

    let markdown_ignore_emphasis = markdown_content(
        aget_home,
        site,
        "/markdown-inline-blocks",
        &[("crawl4ai.ignore_emphasis", "true")],
    );
    assert!(markdown_ignore_emphasis
        .contains("Status: ~~removed~~, soft, under, `Cmd K`, `TTY`, \"quoted\""));
    assert!(markdown_ignore_emphasis.contains("Definition with detail."));
    assert!(!markdown_ignore_emphasis.contains("_soft_"));
    assert!(!markdown_ignore_emphasis.contains("_under_"));
    assert!(!markdown_ignore_emphasis.contains("**detail**"));
    assert!(markdown_ignore_emphasis.contains("~~removed~~"));

    let markdown_hide_strikethrough = markdown_content(
        aget_home,
        site,
        "/markdown-inline-blocks",
        &[("crawl4ai.hide_strikethrough", "true")],
    );
    assert!(markdown_hide_strikethrough.contains("Status: ~~removed~~, _soft_, _under_"));
    assert!(!markdown_hide_strikethrough.contains("Styled removed text."));

    let markdown_emphasis_markers = markdown_content(
        aget_home,
        site,
        "/markdown-inline-blocks",
        &[
            ("crawl4ai.emphasis_mark", "*"),
            ("crawl4ai.strong_mark", "__"),
        ],
    );
    assert!(markdown_emphasis_markers.contains("Status: ~~removed~~, *soft*, *under*,"));
    assert!(markdown_emphasis_markers.contains("Definition with __detail__."));
    assert!(!markdown_emphasis_markers.contains("_soft_"));
    assert!(!markdown_emphasis_markers.contains("**detail**"));

    let markdown_quote_markers = markdown_content(
        aget_home,
        site,
        "/markdown-inline-blocks",
        &[
            ("crawl4ai.open_quote", "<<"),
            ("crawl4ai.close_quote", ">>"),
        ],
    );
    assert!(markdown_quote_markers.contains("`Cmd K`, `TTY`, <<quoted>>, HTML"));

    let markdown_escape_snob = markdown_content(
        aget_home,
        site,
        "/markdown-escape-snob",
        &[("crawl4ai.escape_snob", "true")],
    );
    assert_eq!(
        markdown_escape_snob,
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

    let markdown_unicode_default = markdown_content(aget_home, site, "/markdown-unicode-snob", &[]);
    assert_eq!(
        markdown_unicode_default,
        "# Unicode Snob\n\n© — “quote” → ← · œuvre café."
    );

    let markdown_unicode_ascii = markdown_content(
        aget_home,
        site,
        "/markdown-unicode-snob",
        &[("crawl4ai.unicode_snob", "false")],
    );
    assert_eq!(
        markdown_unicode_ascii,
        "# Unicode Snob\n\n(C) -- \"quote\" -> <- * oeuvre cafe."
    );

    let markdown_body_width = markdown_content(
        aget_home,
        site,
        "/markdown-body-width",
        &[("crawl4ai.body_width", "24")],
    );
    assert_eq!(
        markdown_body_width,
        concat!(
            "# Body Width\n\n",
            "Alpha beta gamma delta\n",
            "epsilon zeta eta theta\n",
            "iota kappa lambda.\n\n",
            "* List item should stay on one rendered line even when the width is narrow.\n\n",
            "```\n",
            "code line should not wrap when body width is narrow\n",
            "```"
        )
    );

    let markdown_wrap_list_items = markdown_content(
        aget_home,
        site,
        "/markdown-body-width",
        &[
            ("crawl4ai.body_width", "24"),
            ("crawl4ai.wrap_list_items", "true"),
        ],
    );
    assert_eq!(
        markdown_wrap_list_items,
        concat!(
            "# Body Width\n\n",
            "Alpha beta gamma delta\n",
            "epsilon zeta eta theta\n",
            "iota kappa lambda.\n\n",
            "* List item should stay\n",
            "on one rendered line\n",
            "even when the width is\n",
            "narrow.\n\n",
            "```\n",
            "code line should not wrap when body width is narrow\n",
            "```"
        )
    );

    let markdown_wrap_tables_default = markdown_content(
        aget_home,
        site,
        "/markdown-wrap-tables",
        &[("crawl4ai.body_width", "24")],
    );
    assert_eq!(
        markdown_wrap_tables_default,
        concat!(
            "# Wrap Tables\n\n",
            "| Name | Value |\n",
            "| --- | --- |\n",
            "| Alpha | one two three four five six seven eight |"
        )
    );

    let markdown_pad_tables = markdown_content(
        aget_home,
        site,
        "/markdown-wrap-tables",
        &[("crawl4ai.pad_tables", "true")],
    );
    assert_eq!(
        markdown_pad_tables,
        concat!(
            "# Wrap Tables\n\n",
            "| Name  | Value                                   |\n",
            "| ------| ----------------------------------------|\n",
            "| Alpha | one two three four five six seven eight |"
        )
    );

    let markdown_wrap_tables_true = markdown_content(
        aget_home,
        site,
        "/markdown-wrap-tables",
        &[
            ("crawl4ai.body_width", "24"),
            ("crawl4ai.wrap_tables", "true"),
        ],
    );
    assert_eq!(
        markdown_wrap_tables_true,
        concat!(
            "# Wrap Tables\n\n",
            "| Name | Value |\n",
            "| --- | --- |\n",
            "| Alpha | one two three\n",
            "four five six seven\n",
            "eight |"
        )
    );

    let markdown_wrap_links_default = markdown_content(
        aget_home,
        site,
        "/markdown-wrap-links",
        &[("crawl4ai.body_width", "34")],
    );
    assert_ne!(
        markdown_wrap_links_default,
        format!(
            "# Wrap Links\n\nAlpha beta [docs link]({}) gamma delta epsilon zeta eta theta.",
            site.url("/docs")
        )
    );

    let markdown_wrap_links_false = markdown_content(
        aget_home,
        site,
        "/markdown-wrap-links",
        &[
            ("crawl4ai.body_width", "34"),
            ("crawl4ai.wrap_links", "false"),
        ],
    );
    assert_eq!(
        markdown_wrap_links_false,
        format!(
            "# Wrap Links\n\nAlpha beta [docs link]({}) gamma delta epsilon zeta eta theta.",
            site.url("/docs")
        )
    );

    let markdown_single_line_break = markdown_content(
        aget_home,
        site,
        "/markdown-single-line-break",
        &[("crawl4ai.single_line_break", "true")],
    );
    assert_eq!(
        markdown_single_line_break,
        concat!(
            "# Single Line Break\n",
            "First paragraph.\n",
            "Second paragraph with `inline` code.\n",
            "```\n",
            "first\n",
            "\n",
            "second\n",
            "```"
        )
    );
}
