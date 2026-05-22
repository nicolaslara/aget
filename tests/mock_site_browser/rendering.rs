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
        "Shadow Shell Shadow Title Projected Detail"
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
        .backend_option("crawl4ai.delay_before_return_html", "0.4")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Slow Client Shell Slow Client Rendered");
}
