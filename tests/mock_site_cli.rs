mod support;

#[path = "mock_site_cli/owned.rs"]
mod owned;
#[path = "mock_site_cli/owned_site.rs"]
mod owned_site;

use aget::OutputFormat;
use assert_cmd::Command;
use support::mock_site::{MockResponse, MockSite};
use support::mock_site_cli::{aget, mock_backend_command, success_data};

#[test]
fn mock_site_fetch_handles_redirect_output_shaping_and_waits() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/redirect"),
            "--content-format",
            "text",
            "--selector",
            "main",
            "--exclude-selector",
            "nav",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["final_url"], site.url("/public"));
    assert!(json["content"].as_str().unwrap().contains("Public Main"));
    assert!(!json["content"]
        .as_str()
        .unwrap()
        .contains("In-Article Navigation"));

    let delayed = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/delayed"),
            "--content-format",
            "text",
            "--wait-for-selector",
            "#ready",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let delayed_json = success_data(&delayed, "get");
    assert!(delayed_json["content"]
        .as_str()
        .unwrap()
        .contains("Delayed Ready"));
}

#[test]
fn default_cli_fetch_uses_owned_backend_without_command_dependencies() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env_remove("AGET_CRAWL4AI_COMMAND")
        .env_remove("AGET_AGENT_BROWSER_COMMAND")
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/public"),
            "--content-format",
            "text",
            "--selector",
            "main",
            "--exclude-selector",
            "nav",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["extractor"], "aget-owned-extractor");
    assert_eq!(json["content"], "Public Main Visible public article.");
}

#[test]
fn backend_parity_covers_extractor_content_formats() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command();
    let site = MockSite::builder()
        .route(
            "/formats",
            MockResponse::html(
                r#"
<html>
  <body>
    <main>
      <h1>Format Heading</h1>
      <p>Format body text.</p>
    </main>
  </body>
</html>
"#,
            ),
        )
        .start();

    let markdown = aget(&aget_home, &fake_backend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(markdown.content_format, "markdown");
    assert!(markdown.content.contains("Format Heading"));
    assert!(markdown.content.contains("Format body text."));

    let text = aget(&aget_home, &fake_backend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(text.content_format, "text");
    assert_eq!(text.content, "Format Heading Format body text.");

    let html = aget(&aget_home, &fake_backend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert_eq!(html.content_format, "html");
    assert!(html.content.contains("<main>"));
    assert!(html.content.contains("<h1>Format Heading</h1>"));

    let json = aget(&aget_home, &fake_backend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Json)
        .exclude_selector("p.ad")
        .run()
        .unwrap();
    assert_eq!(json.content_format, "json");
    let parsed: serde_json::Value = serde_json::from_str(&json.content).unwrap();
    assert_eq!(parsed["url"], site.url("/formats"));
    assert_eq!(parsed["content"], "Format Heading Format body text.");
}
