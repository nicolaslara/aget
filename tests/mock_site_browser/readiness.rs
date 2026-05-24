use std::time::Duration;

use aget::{Aget, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site::{MockResponse, MockSite};

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_extractor_backend_removes_rendered_style_overlays_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/rendered-style-overlay",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Rendered Overlay Story</h1>
      <div id="style-overlay" style="position: fixed; inset: 0; z-index: 10000; background: rgba(0, 0, 0, 0.8);">
        <p>Style overlay text.</p>
      </div>
      <p>Useful rendered article text.</p>
    </main>
    <script>
      document.body.dataset.rendered = "true"
    </script>
  </body>
</html>
"##,
            ),
        )
        .start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/rendered-style-overlay"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(
        extraction.content,
        "Rendered Overlay Story\nUseful rendered article text."
    );
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_extractor_backend_honors_wait_for_images_option_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/image-wait",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Image Shell</h1>
      <p id="image-status">Image Pending</p>
      <img id="slow-image" src="/slow-image.svg" alt="slow">
    </main>
    <script>
      const status = document.querySelector("#image-status")
      const image = document.querySelector("#slow-image")
      image.addEventListener("load", () => { status.textContent = "Image Loaded" })
      image.addEventListener("error", () => { status.textContent = "Image Failed" })
    </script>
  </body>
</html>
"##,
            ),
        )
        .route(
            "/slow-image.svg",
            MockResponse::html(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"></svg>"#,
            )
            .content_type("image/svg+xml")
            .delay(Duration::from_millis(250)),
        )
        .start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/image-wait"))
        .content_format(OutputFormat::Text)
        .backend_option("aget.wait_until", "domcontentloaded")
        .backend_option("aget.delay_before_return_html", "0")
        .backend_option("aget.wait_for_images", "true")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Image Shell\nImage Loaded");
    assert!(extraction.warnings.is_empty());
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_extractor_backend_honors_networkidle_wait_until_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/networkidle",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Network Shell</h1>
      <p id="network-result">Loading</p>
    </main>
    <script>
      fetch("/slow-fragment")
        .then((response) => response.text())
        .then((text) => { document.querySelector("#network-result").textContent = text })
    </script>
  </body>
</html>
"##,
            ),
        )
        .route(
            "/slow-fragment",
            MockResponse::html("Network Settled")
                .content_type("text/plain")
                .delay(Duration::from_millis(650)),
        )
        .start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/networkidle"))
        .content_format(OutputFormat::Text)
        .backend_option("aget.wait_until", "networkidle")
        .backend_option("aget.delay_before_return_html", "0")
        .backend_option("aget.page_timeout", "5000")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Network Shell\nNetwork Settled");
}
