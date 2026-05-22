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
            "/metadata",
            MockResponse::html(
                r#"
<html>
  <head>
    <title>Metadata Title</title>
    <meta name="description" content="Metadata description">
    <meta name="keywords" content="agent,context,local">
    <meta name="author" content="Aget Docs">
    <meta property="og:title" content="Open Graph Title">
    <meta property="og:description" content="Open Graph description">
    <meta name="twitter:title" content="Twitter Title">
    <meta property="article:published_time" content="2026-05-22T10:00:00Z">
  </head>
  <body>
    <main class="article">
      <h1>Metadata Body</h1>
      <p>Metadata body text.</p>
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
