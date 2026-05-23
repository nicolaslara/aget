use aget::{Aget, AgetExtractorBackend, OutputFormat};

use crate::support::mock_site::{MockResponse, MockSite};

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_extractor_backend_flattens_shadow_dom_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/shadow-dom",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Shadow Shell</h1>
      <shadow-card><span slot="detail">Projected Detail</span></shadow-card>
    </main>
    <script>
      customElements.define("shadow-card", class extends HTMLElement {
        constructor() {
          super()
          const root = this.attachShadow({ mode: "closed" })
          root.innerHTML = "<style>p{color:red}</style><article><h2>Shadow Title</h2><slot name=\"detail\">Fallback Detail</slot></article>"
        }
      })
    </script>
  </body>
</html>
"##,
            ),
        )
        .start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/shadow-dom"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.flatten_shadow_dom", "true")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(
        extraction.content,
        "Shadow Shell\nShadow Title\nProjected Detail"
    );
    assert!(extraction.warnings.is_empty());
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_extractor_backend_processes_accessible_iframes_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/iframe-page",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Frame Shell</h1>
      <iframe src="/iframe-inner"></iframe>
    </main>
  </body>
</html>
"##,
            ),
        )
        .route(
            "/iframe-inner",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h2>Frame Title</h2>
      <p>Frame Body</p>
    </main>
  </body>
</html>
"##,
            ),
        )
        .start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .get(site.url("/iframe-page"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.process_iframes", "true")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert!(
        extraction.content.contains("Frame Title"),
        "content: {:?}; warnings: {:?}",
        extraction.content,
        extraction.warnings
    );
    assert!(
        extraction.content.contains("Frame Body"),
        "content: {:?}; warnings: {:?}",
        extraction.content,
        extraction.warnings
    );
    assert!(extraction.warnings.is_empty());
}
