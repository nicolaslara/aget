use aget::{Aget, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site::{MockResponse, MockSite};

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_extractor_backend_renders_waited_javascript_page_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/delayed"))
        .content_format(OutputFormat::Text)
        .wait_for_selector("#ready")
        .backend_option("crawl4ai.page_timeout", "5000")
        .backend_option("crawl4ai.wait_for_timeout", "1000")
        .backend_option("crawl4ai.wait_until", "domcontentloaded")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert!(extraction.content.contains("Delayed Ready"));
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_extractor_backend_renders_scripted_page_without_wait_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/client-rendered",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Client Shell</h1>
      <div id="client-result">Loading</div>
    </main>
    <script>
      setTimeout(() => {
        document.querySelector("#client-result").innerHTML = "<p>Client Rendered</p>"
      }, 25)
    </script>
  </body>
</html>
"##,
            ),
        )
        .start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/client-rendered"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Client Shell\nClient Rendered");
}
