use std::fs;

use aget::OutputFormat;

use crate::support::mock_site::MockSite;
use crate::support::mock_site_cli::{aget, mock_backend_command};

#[test]
fn documents_output_limits_out_file_and_warning_contract() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    let out_path = temp.path().join("agent-context.txt");

    let result = aget(&aget_home, &fake_backend)
        .get(site.url("/warning"))
        .content_format(OutputFormat::Text)
        .output(&out_path)
        .max_chars(12)
        .run()
        .unwrap();

    assert_eq!(result.warnings, vec!["mock warning"]);
    assert_eq!(result.content, "Warning Page");
    assert_eq!(result.artifacts.content, out_path.to_string_lossy());
    assert_eq!(fs::read_to_string(&out_path).unwrap(), "Warning Page\n");
    assert_eq!(result.limits.max_chars, Some(12));
    assert!(result.limits.truncated);
    assert_eq!(result.limits.truncated_by.as_deref(), Some("max_chars"));
    assert!(result.limits.content_chars_before_truncation > 12);
    assert_eq!(result.limits.content_chars_after_truncation, 12);
}
