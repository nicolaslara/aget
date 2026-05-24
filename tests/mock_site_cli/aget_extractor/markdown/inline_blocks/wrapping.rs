use std::path::Path;

use crate::support::mock_site::MockSite;

use super::super::markdown_content;

pub(super) fn assert_wrapping_options(aget_home: &Path, site: &MockSite) {
    let markdown_body_width = markdown_content(
        aget_home,
        site,
        "/markdown-body-width",
        &[("aget.body_width", "24")],
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
        &[("aget.body_width", "24"), ("aget.wrap_list_items", "true")],
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
        &[("aget.body_width", "24")],
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
        &[("aget.pad_tables", "true")],
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
        &[("aget.body_width", "24"), ("aget.wrap_tables", "true")],
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
        &[("aget.body_width", "34")],
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
        &[("aget.body_width", "34"), ("aget.wrap_links", "false")],
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
        &[("aget.single_line_break", "true")],
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
