mod support;

use std::time::Duration;

use aget::{Aget, AgetBrowserBackend, AgetExtractorBackend, OutputFormat};
use support::mock_site::{MockResponse, MockSite};
use support::mock_site_cli::{
    save_cookie_session, save_storage_session, storage_rendered_site, FailingExtractor,
};

#[test]
fn aget_browser_fallback_replays_cookie_backed_session_without_agent_browser() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let fallback = Aget::new(&aget_home)
        .with_extractor_backend(FailingExtractor)
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .get(site.url("/protected"))
        .session("app")
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();

    assert_eq!(fallback.extractor, "aget-owned-browser-fallback");
    assert_eq!(fallback.content, "Protected Account Private account body.");
    assert_eq!(
        fallback.warnings,
        vec!["aget-owned fallback used after primary extractor failed"]
    );
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_browser_fallback_renders_cookie_backed_scripted_page_with_chrome() {
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
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let fallback = Aget::new(&aget_home)
        .with_extractor_backend(FailingExtractor)
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .get(site.url("/client-rendered"))
        .session("app")
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();

    assert_eq!(fallback.extractor, "aget-owned-browser-fallback");
    assert_eq!(fallback.content, "Client Shell Client Rendered");
    assert!(site.received_cookie("/client-rendered", "app_session", "valid-app"));
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_browser_fallback_renders_local_storage_backed_session_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = storage_rendered_site();
    save_storage_session(
        &aget_home,
        "storage",
        &site.origin(),
        "local_token",
        "storage-secret",
    );

    let fallback = Aget::new(&aget_home)
        .with_extractor_backend(FailingExtractor)
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .get(site.url("/storage-rendered"))
        .session("storage")
        .content_format(OutputFormat::Text)
        .wait_for_selector("#storage-result h1")
        .run()
        .unwrap();

    assert_eq!(fallback.extractor, "aget-owned-browser-fallback");
    assert_eq!(fallback.content, "Storage App Shell Storage Protected");
    assert!(site.received_header("/storage-api", "x-local-token", "storage-secret"));
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn aget_extractor_backend_renders_local_storage_backed_session_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = storage_rendered_site();
    save_storage_session(
        &aget_home,
        "storage",
        &site.origin(),
        "local_token",
        "storage-secret",
    );

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(AgetExtractorBackend::default())
        .with_browser_automation_backend(AgetBrowserBackend::default())
        .get(site.url("/storage-rendered"))
        .session("storage")
        .content_format(OutputFormat::Text)
        .wait_for_selector("#storage-result h1")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Storage App Shell Storage Protected");
    assert!(site.received_header("/storage-api", "x-local-token", "storage-secret"));
}

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
    assert_eq!(extraction.content, "Client Shell Client Rendered");
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
        "Rendered Overlay Story Useful rendered article text."
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
        .backend_option("crawl4ai.wait_until", "domcontentloaded")
        .backend_option("crawl4ai.delay_before_return_html", "0")
        .backend_option("crawl4ai.wait_for_images", "true")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Image Shell Image Loaded");
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
        .backend_option("crawl4ai.wait_until", "networkidle")
        .backend_option("crawl4ai.delay_before_return_html", "0")
        .backend_option("crawl4ai.page_timeout", "5000")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Network Shell Network Settled");
}
