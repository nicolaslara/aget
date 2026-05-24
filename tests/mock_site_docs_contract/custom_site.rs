use aget::OutputFormat;

use crate::support::mock_site::{MockResponse, MockSite};
use crate::support::mock_site_cli::aget;

#[test]
fn documents_custom_site_routes_for_extraction_features() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/guide",
            MockResponse::html(
                r#"
<html>
  <body>
    <main>
      <h1>Custom Guide</h1>
      <p>Feature-specific extraction fixture.</p>
      <aside>Remove this sidebar</aside>
    </main>
  </body>
</html>
"#,
            )
            .header("X-Fixture", "custom-guide"),
        )
        .route("/guide/latest", MockResponse::redirect("/guide"))
        .start();

    let result = aget(&aget_home)
        .get(site.url("/guide/latest"))
        .content_format(OutputFormat::Text)
        .selector("main")
        .run()
        .unwrap();

    assert_eq!(result.final_url, site.url("/guide"));
    assert_eq!(
        result.content,
        "Custom Guide\nFeature-specific extraction fixture.\nRemove this sidebar"
    );
}
