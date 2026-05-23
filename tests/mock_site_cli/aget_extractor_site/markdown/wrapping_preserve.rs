use crate::support::mock_site::{MockResponse, MockSiteBuilder};

pub(crate) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    builder
        .route(
            "/markdown-wrap-links",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Wrap Links</h1>
      <p>Alpha beta <a href="/docs">docs link</a> gamma delta epsilon zeta eta theta.</p>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/markdown-reference-paragraphs",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Reference Paragraphs</h1>
      <p>First <a href="/alpha">alpha</a> and <a href="/beta">beta</a>.</p>
      <p>Second <a href="/alpha">alpha again</a>.</p>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/markdown-wrap-tables",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Wrap Tables</h1>
      <table>
        <thead><tr><th>Name</th><th>Value</th></tr></thead>
        <tbody><tr><td>Alpha</td><td>one two three four five six seven eight</td></tr></tbody>
      </table>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/markdown-body-width",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Body Width</h1>
      <p>Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda.</p>
      <ul><li>List item should stay on one rendered line even when the width is narrow.</li></ul>
      <pre><code>code line should not wrap when body width is narrow</code></pre>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/markdown-single-line-break",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Single Line Break</h1>
      <p>First paragraph.</p>
      <p>Second paragraph with <code>inline</code> code.</p>
      <pre><code>first

second</code></pre>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/markdown-preserve-tags",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Preserve Tags</h1>
      <p>Before <custom-card><strong>Raw</strong><span> HTML</span></custom-card> after.</p>
      <p>Second <math-box><em>Equation</em></math-box> done.</p>
    </main>
  </body>
</html>
"#,
            ),
        )
}
