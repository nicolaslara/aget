use crate::support::mock_site::{MockResponse, MockSiteBuilder};

pub(crate) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    builder
        .route(
            "/markdown-nested-lists",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Nested Steps</h1>
      <ol>
        <li>Install
          <ul>
            <li>Open settings</li>
            <li>Confirm access</li>
          </ul>
        </li>
        <li>Run fetch</li>
      </ol>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/markdown-links",
            MockResponse::html(
                r##"
<html>
  <body>
    <main class="article">
      <h1>Link Defaults</h1>
      <a href="/linked-heading" title="Heading title"><h2>Linked Heading</h2></a>
      <p>Read <a href="/guide" title="Guide &quot;title&quot; [v1] (draft)">the guide</a> or <a href="mailto:help@example.com">email support</a>.</p>
      <p>Canonical <a href="https://example.com/docs" title="Docs title">https://example.com/docs</a>.</p>
      <p>Jump <a href="#details">within page</a>.</p>
      <p>Link label <a href="/api"><code>API v1</code></a> and standalone <code>inline_code</code>.</p>
      <p>Empty <a href="/empty"></a> marker.</p>
      <p>Asset <a href="/release(2026)">release notes</a> and <img src="/assets/diagram(1).png" alt="A [diagram] (v1)" width="640" height="360">.</p>
      <p>Icon <a href="/download"><img src="/icons/app(1).svg" alt="Download [app]" width="32"></a>.</p>
      <p>Missing <img src="/missing-alt.png"> and empty <img src="/empty-alt.png" alt="">.</p>
    </main>
  </body>
</html>
"##,
            ),
        )
        .route(
            "/markdown-code-whitespace",
            MockResponse::html(
                "<html><body><main class=\"article\"><h1>Code Whitespace</h1><pre><code>first\nlet padded = true;  \n\nlast</code></pre></main></body></html>",
            ),
        )
        .route(
            "/markdown-ordered-start",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Ordered Start</h1>
      <ol start="4">
        <li>Resume</li>
        <li>Verify</li>
      </ol>
      <ol start="later">
        <li>Fallback</li>
      </ol>
    </main>
  </body>
</html>
"#,
            ),
        )
}
