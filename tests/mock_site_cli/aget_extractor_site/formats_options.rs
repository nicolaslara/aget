use crate::support::mock_site::{MockResponse, MockSiteBuilder};

pub(crate) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    builder
        .route(
            "/formats",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Format Heading</h1>
      <p>Format body text.</p>
      <p class="ad">Promotional aside.</p>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/word-threshold",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Threshold Example Title</h1>
      <p>Keep this paragraph because it has enough useful words.</p>
      <p class="caption">Tiny caption</p>
      <p>Short link</p>
      <pre><code><span> </span></code></pre>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/wait-ready",
            MockResponse::html(
                r#"<html><body><main><div id="ready">Ready Now</div></main></body></html>"#,
            ),
        )
}
