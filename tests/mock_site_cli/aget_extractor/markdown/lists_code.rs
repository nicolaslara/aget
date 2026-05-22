use std::path::Path;

use crate::support::mock_site::MockSite;

use super::markdown_content;

pub(super) fn assert_lists_and_code_blocks(aget_home: &Path, site: &MockSite) {
    let markdown_nested_lists = markdown_content(aget_home, site, "/markdown-nested-lists", &[]);
    assert_eq!(
        markdown_nested_lists,
        "# Nested Steps\n\n1. Install\n  * Open settings\n  * Confirm access\n2. Run fetch"
    );

    let markdown_unordered_marker = markdown_content(
        aget_home,
        site,
        "/markdown-nested-lists",
        &[("crawl4ai.ul_item_mark", "-")],
    );
    assert_eq!(
        markdown_unordered_marker,
        "# Nested Steps\n\n1. Install\n  - Open settings\n  - Confirm access\n2. Run fetch"
    );

    let markdown_google_doc_lists_default =
        markdown_content(aget_home, site, "/markdown-google-doc-lists", &[]);
    assert_eq!(
        markdown_google_doc_lists_default,
        "# Google Doc Lists\n\n* First step\n* Nested step\n\n* Nested bullet"
    );

    let markdown_google_doc_lists = markdown_content(
        aget_home,
        site,
        "/markdown-google-doc-lists",
        &[("crawl4ai.google_doc", "true")],
    );
    assert_eq!(
        markdown_google_doc_lists,
        "# Google Doc Lists\n\n1. First step\n    2. Nested step\n\n  * Nested bullet"
    );

    let markdown_google_doc_custom_indent = markdown_content(
        aget_home,
        site,
        "/markdown-google-doc-lists",
        &[
            ("crawl4ai.google_doc", "true"),
            ("crawl4ai.google_list_indent", "72"),
        ],
    );
    assert_eq!(
        markdown_google_doc_custom_indent,
        "# Google Doc Lists\n\n1. First step\n  2. Nested step\n\n* Nested bullet"
    );

    let markdown_code_whitespace =
        markdown_content(aget_home, site, "/markdown-code-whitespace", &[]);
    assert_eq!(
        markdown_code_whitespace,
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

    let markdown_mark_code_false = markdown_content(
        aget_home,
        site,
        "/markdown-code-whitespace",
        &[("crawl4ai.mark_code", "false")],
    );
    assert_eq!(markdown_mark_code_false, markdown_code_whitespace);

    let markdown_ordered_start = markdown_content(aget_home, site, "/markdown-ordered-start", &[]);
    assert_eq!(
        markdown_ordered_start,
        "# Ordered Start\n\n1. Resume\n2. Verify\n\n1. Fallback"
    );
}
