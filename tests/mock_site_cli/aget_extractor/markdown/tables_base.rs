use std::path::Path;

use crate::support::mock_site::MockSite;

use super::markdown_content;

pub(super) fn assert_tables_and_base_links(aget_home: &Path, site: &MockSite) {
    let markdown = markdown_content(aget_home, site, "/markdown", &[]);
    assert_eq!(
        markdown,
        format!(
            "# Guide\n\nIntro with **bold** and [docs]({}).\n\nLine one  \nLine two\n\n* First item\n* Second `code`\n\nData Table\n\n| Name | Value |\n| --- | --- |\n| Alpha | [A\\|1]({}) |\n\n```\nlet answer = 42;\n```",
            site.url("/docs"),
            site.url("/alpha")
        )
    );

    let markdown_ignore_tables = markdown_content(
        aget_home,
        site,
        "/markdown",
        &[("crawl4ai.ignore_tables", "true")],
    );
    assert_eq!(
        markdown_ignore_tables,
        format!(
            "# Guide\n\nIntro with **bold** and [docs]({}).\n\nLine one  \nLine two\n\n* First item\n* Second `code`\n\nData Table\nName Value\nAlpha [A|1]({})\n\n```\nlet answer = 42;\n```",
            site.url("/docs"),
            site.url("/alpha")
        )
    );

    let markdown_bypass_tables = markdown_content(
        aget_home,
        site,
        "/markdown",
        &[("crawl4ai.bypass_tables", "true")],
    );
    assert_eq!(
        markdown_bypass_tables,
        format!(
            "# Guide\n\nIntro with **bold** and [docs]({}).\n\nLine one  \nLine two\n\n* First item\n* Second `code`\n\n<table>Data Table\n<tr><th>Name</th><th>Value</th></tr><tr><td>Alpha</td><td>[A|1]({})</td></tr></table>\n\n```\nlet answer = 42;\n```",
            site.url("/docs"),
            site.url("/alpha")
        )
    );

    let markdown_base = markdown_content(aget_home, site, "/markdown-base", &[]);
    assert_eq!(
        markdown_base,
        format!(
            "# Base Links\n\nRead the [base page]({}).",
            site.url("/guide/page.html")
        )
    );
}
