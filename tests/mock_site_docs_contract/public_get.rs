use std::fs;
use std::path::PathBuf;

use aget::OutputFormat;

use crate::support::mock_site::MockSite;
use crate::support::mock_site_cli::{aget, mock_backend_command};

#[test]
fn documents_public_get_json_contract_for_agents() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();

    let result = aget(&aget_home, &fake_backend)
        .get(site.url("/public"))
        .content_format(OutputFormat::Text)
        .selector("main")
        .exclude_selector("nav")
        .run()
        .unwrap();

    assert_eq!(result.url, site.url("/public"));
    assert_eq!(result.final_url, site.url("/public"));
    assert_eq!(result.content_format, "text");
    assert_eq!(result.extractor, "crawl4ai");
    assert_eq!(result.content, "Public Main Visible public article.");
    assert!(result.sessions.is_empty());
    assert!(!result.sensitive);
    assert!(result.warnings.is_empty());
    assert!(result.timing_ms.total > 0);
    assert_eq!(result.limits.max_chars, None);
    assert!(!result.limits.truncated);
    assert_eq!(result.output_options.content_format, OutputFormat::Text);
    assert_eq!(result.output_options.selector.as_deref(), Some("main"));
    assert_eq!(
        result.output_options.exclude_selector.as_deref(),
        Some("nav")
    );
    assert!(result.output_options.backend_options.is_empty());

    let content_path = PathBuf::from(&result.artifacts.content);
    let metadata_path = PathBuf::from(&result.artifacts.metadata);
    assert!(content_path.starts_with(aget_home.join("runs")));
    assert!(metadata_path.starts_with(aget_home.join("runs")));
    assert_eq!(
        fs::read_to_string(content_path).unwrap(),
        "Public Main Visible public article.\n"
    );

    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["url"], site.url("/public"));
    assert_eq!(metadata["content_format"], "text");
    assert_eq!(metadata["sensitive"], false);
}
