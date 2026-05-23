#[path = "inline_blocks/escape_unicode.rs"]
mod escape_unicode;
#[path = "inline_blocks/google_preserve.rs"]
mod google_preserve;
#[path = "inline_blocks/semantics.rs"]
mod semantics;
#[path = "inline_blocks/wrapping.rs"]
mod wrapping;

use std::path::Path;

use crate::support::mock_site::MockSite;

pub(super) fn assert_inline_blocks(aget_home: &Path, site: &MockSite) {
    semantics::assert_semantic_inline_blocks(aget_home, site);
    escape_unicode::assert_escaping_and_unicode_options(aget_home, site);
    google_preserve::assert_google_doc_and_preserve_tags(aget_home, site);
    wrapping::assert_wrapping_options(aget_home, site);
}
