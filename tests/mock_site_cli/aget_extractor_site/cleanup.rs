use crate::support::mock_site::{MockResponse, MockSiteBuilder};

pub(crate) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    builder.route(
        "/html-cleanup",
        MockResponse::html(
            r#"
<html>
  <head>
    <title>Cleanup Title</title>
    <meta name="description" content="private metadata">
    <link rel="canonical" href="/canonical">
    <style>.hidden { display: none; }</style>
  </head>
  <body>
    <main id="cleanup-main" class="article" data-private="main-secret" style="color:red">
      <h1>Cleanup Main</h1>
      <!-- debug-secret should not survive owned cleanup -->
      <p data-select="summary" data-private="paragraph-secret" style="color:blue" onclick="steal()" aria-label="private label">Visible body.</p>
      <a id="kept-link" class="cta" href="/kept" title="Kept title" rel="nofollow" data-private="link-secret">Kept link</a>
      <a id="external-link" href="https://external.example/out">External link</a>
      <a id="social-link" href="https://www.linkedin.com/company/aget">Social link</a>
      <a id="custom-social-link" href="https://social.example/company/aget">Custom social link</a>
      <img id="diagram" class="figure" src="/diagram.png" alt="Diagram" width="640" height="480" data-private="image-secret" style="display:none">
      <img id="remote-image" src="https://cdn.example/remote.png" alt="Remote image">
      <img id="social-image" src="https://www.linkedin.com/logo.png" alt="Social image">
      <img id="inline-image" src="data:image/png;base64,QUJDRA==" alt="Inline image">
      <section id="empty-wrapper"><span id="empty-span"></span></section>
      <a id="empty-anchor" href="/empty"></a>
      <br id="empty-break">
      <table><tbody><tr id="empty-row"><td id="empty-cell"></td></tr></tbody></table>
      <pre><code><span id="code-space"> </span></code></pre>
      <meta name="body-meta" content="remove me">
      <link rel="preload" href="/asset.css">
      <script>window.secret = "remove me";</script>
      <noscript>Remove fallback text</noscript>
    </main>
  </body>
</html>
"#,
        ),
    )
}
