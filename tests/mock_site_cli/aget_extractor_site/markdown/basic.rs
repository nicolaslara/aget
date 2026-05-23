use crate::support::mock_site::{MockResponse, MockSiteBuilder};

pub(crate) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    builder
        .route(
            "/markdown",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Guide</h1>
      <p>Intro with <span><strong>bold</strong></span> and <a href="/docs">docs</a>.</p>
      <p>Line one<br>Line two</p>
      <ul>
        <li>First item</li>
        <li>Second <code>code</code></li>
      </ul>
      <table>
        <caption>Data Table</caption>
        <thead><tr><th>Name</th><th>Value</th></tr></thead>
        <tbody><tr><td>Alpha</td><td><a href="/alpha">A|1</a></td></tr></tbody>
      </table>
      <pre><code>let answer = 42;</code></pre>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/markdown-base",
            MockResponse::html(
                r#"
<html>
  <head><base href="/guide/"></head>
  <body>
    <main class="article">
      <h1>Base Links</h1>
      <p>Read the <a href="page.html">base page</a>.</p>
    </main>
  </body>
</html>
"#,
            ),
        )
}
