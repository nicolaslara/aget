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

    let markdown_ordered_start = markdown_content(aget_home, site, "/markdown-ordered-start", &[]);
    assert_eq!(
        markdown_ordered_start,
        "# Ordered Start\n\n1. Resume\n2. Verify\n\n1. Fallback"
    );
}
