use aget::{Aget, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site::{MockResponse, MockSite};

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_extractor_backend_honors_render_delay_option_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/slow-client-rendered",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Slow Client Shell</h1>
      <div id="client-result">Loading</div>
    </main>
    <script>
      setTimeout(() => {
        document.querySelector("#client-result").innerHTML = "<p>Slow Client Rendered</p>"
      }, 200)
    </script>
  </body>
</html>
"##,
            ),
        )
        .start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/slow-client-rendered"))
        .content_format(OutputFormat::Text)
        .backend_option("aget.delay_before_return_html", "0.4")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(
        extraction.content,
        "Slow Client Shell\nSlow Client Rendered"
    );
}
