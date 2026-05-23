use std::path::Path;

use crate::support::mock_site::MockSite;

use super::super::markdown_content;

pub(super) fn assert_semantic_inline_blocks(aget_home: &Path, site: &MockSite) {
    let markdown_inline_blocks = markdown_content(aget_home, site, "/markdown-inline-blocks", &[]);
    assert_eq!(
        markdown_inline_blocks,
        format!(
            "# Reference Bits\n\nStatus: ~~removed~~, _soft_, _under_, `Cmd K`, `TTY`, \"quoted\", HTML, power 2, and water 2.\n\nStyled removed text.\n\n1\\. Not a generated list.\n\n\\- Not a generated bullet.\n\n\\+ Not a generated plus bullet.\n\nLiteral \\\\*stars\\\\* and \\\\[brackets\\\\].\n\n* * *\n\n> Quoted **block**.\n>\n> Second line.\n\n![Figure alt]({})\n\nFigure caption with source.\n\nExpandable Summary\n\nHidden detail text.\n\nContact the docs team.\n\nTerm\n    Definition with **detail**.\n\n  *[HTML]: HyperText Markup Language",
            site.url("/figure.png")
        )
    );

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
}
