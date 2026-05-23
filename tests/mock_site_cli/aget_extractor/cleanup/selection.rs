use std::path::Path;

use aget::OutputFormat;

use crate::support::mock_site::MockSite;

use super::support::cleanup_content;

pub(super) fn assert_selection_after_attribute_pruning(aget_home: &Path, site: &MockSite) {
    let selected_by_pruned_attr = cleanup_content(
        aget_home,
        site,
        OutputFormat::Text,
        &[],
        Some(r#"[data-select="summary"]"#),
    );
    assert_eq!(selected_by_pruned_attr, "Visible body.");
}
