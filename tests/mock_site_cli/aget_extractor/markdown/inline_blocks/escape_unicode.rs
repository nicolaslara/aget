use std::path::Path;

use crate::support::mock_site::MockSite;

use super::super::markdown_content;

pub(super) fn assert_escaping_and_unicode_options(aget_home: &Path, site: &MockSite) {
    let markdown_line_start_escapes = markdown_content(
        aget_home,
        site,
        "/markdown-inline-blocks",
        &[
            ("aget.escape_dot", "false"),
            ("aget.escape_dash", "false"),
            ("aget.escape_plus", "false"),
        ],
    );
    assert!(markdown_line_start_escapes.contains("\n\n1. Not a generated list.\n\n"));
    assert!(markdown_line_start_escapes.contains("\n\n- Not a generated bullet.\n\n"));
    assert!(markdown_line_start_escapes.contains("\n\n+ Not a generated plus bullet.\n\n"));
    assert!(!markdown_line_start_escapes.contains("1\\. Not a generated list."));
    assert!(!markdown_line_start_escapes.contains("\\- Not a generated bullet."));
    assert!(!markdown_line_start_escapes.contains("\\+ Not a generated plus bullet."));

    let markdown_backslash_escape = markdown_content(
        aget_home,
        site,
        "/markdown-inline-blocks",
        &[("aget.escape_backslash", "false")],
    );
    assert!(markdown_backslash_escape.contains(r"Literal \*stars\* and \[brackets\]."));
    assert!(!markdown_backslash_escape.contains(r"Literal \\*stars\\* and \\[brackets\\]."));

    let markdown_escape_snob = markdown_content(
        aget_home,
        site,
        "/markdown-escape-snob",
        &[("aget.escape_snob", "true")],
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
        &[("aget.unicode_snob", "false")],
    );
    assert_eq!(
        markdown_unicode_ascii,
        "# Unicode Snob\n\n(C) -- \"quote\" -> <- * oeuvre cafe."
    );
}
