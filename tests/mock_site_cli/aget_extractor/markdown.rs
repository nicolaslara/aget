#[path = "markdown/inline_blocks.rs"]
mod inline_blocks;
#[path = "markdown/links_images.rs"]
mod links_images;
#[path = "markdown/lists_code.rs"]
mod lists_code;
#[path = "markdown/tables_base.rs"]
mod tables_base;

use std::path::Path;

use aget::{Aget, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site::MockSite;

pub(super) fn assert_markdown_rendering(aget_home: &Path, site: &MockSite) {
    tables_base::assert_tables_and_base_links(aget_home, site);
    inline_blocks::assert_inline_blocks(aget_home, site);
    lists_code::assert_lists_and_code_blocks(aget_home, site);
    links_images::assert_links_and_images(aget_home, site);
}

fn markdown_content(
    aget_home: &Path,
    site: &MockSite,
    path: &str,
    backend_options: &[(&str, &str)],
) -> String {
    let mut request = Aget::new(aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url(path))
        .content_format(OutputFormat::Markdown)
        .selector("main.article");
    for (key, value) in backend_options {
        request = request.backend_option(*key, *value);
    }
    request.run().unwrap().content
}
