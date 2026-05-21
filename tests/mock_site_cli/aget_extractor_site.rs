use crate::support::mock_site::{MockResponse, MockSite};

pub(crate) fn aget_extractor_parity_site() -> MockSite {
    MockSite::builder()
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
            "/main-content",
            MockResponse::html(
                r#"
<html>
  <body>
    <header>Site Header</header>
    <nav>Global Navigation</nav>
    <main class="story">
      <h1>Main Story</h1>
      <p>Useful body text.</p>
    </main>
    <aside>Sidebar Noise</aside>
    <footer>Footer Noise</footer>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/multiple-articles",
            MockResponse::html(
                r#"
<html>
  <body>
    <header>Site Header</header>
    <article class="promo">
      <h2>Promo Card</h2>
      <p>Short teaser.</p>
      <a href="/signup">Sign up</a>
    </article>
    <article class="story">
      <h1>Deep Story</h1>
      <p>This article has enough useful body text to beat the promotional card.</p>
      <p>It should be selected as the default main content candidate.</p>
    </article>
    <footer>Footer Noise</footer>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/overlay-content",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="story">
      <h1>Overlay Story</h1>
      <div class="cookie-banner">Cookie banner text.</div>
      <section role="dialog">Newsletter modal text.</section>
      <p>Useful article text.</p>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/labeled-content",
            MockResponse::html(
                r#"
<html>
  <body>
    <nav>Global docs nav</nav>
    <div class="layout">
      <section class="promo-sidebar">Related links and promotions</section>
      <div id="article-content" class="story-body">
        <h1>Labeled Story</h1>
        <p>Useful labeled content should win default extraction without an explicit selector.</p>
      </div>
    </div>
    <footer>Global footer links</footer>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/main-content-link-density",
            MockResponse::html(
                r#"
<html>
  <body>
    <article class="story">
      <h1>Related Reading</h1>
      <p>
        <a href="/one">Guide setup reference quickstart migration examples checklist</a>
        <a href="/two">Release notes archive support community forum changelog</a>
        <a href="/three">Pricing signup trial demo contact docs index</a>
        <a href="/four">More linked navigation labels and repeated index terms</a>
      </p>
    </article>
    <div id="content" class="content">
      <h1>Dense Article</h1>
      <p>Dense useful body text should win because it has direct prose instead of mostly navigation links.</p>
      <p>The local scorer should prefer low-link-density content for agent-ready extraction.</p>
    </div>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/main-content-class-id-noise",
            MockResponse::html(
                r#"
<html>
  <body>
    <article id="comments-panel" class="content story">
      <h1>Community Comments</h1>
      <p>This discussion thread has many words and looks like content, but the class and id mark it as comments noise.</p>
      <p>Additional replies mention setup docs migration release examples and troubleshooting to make the block deceptively dense.</p>
    </article>
    <div id="content" class="story-body">
      <h1>Primary Article</h1>
      <p>The primary article should win even when a noisy comments block has enough text to look important.</p>
    </div>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/selector-miss",
            MockResponse::html(
                r#"
<html>
  <body>
    <header>Selector Header</header>
    <main>
      <h1>Selector Main</h1>
      <p>Selector body text.</p>
    </main>
    <footer>Selector Footer</footer>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/selector-multiple",
            MockResponse::html(
                r#"
<html>
  <body>
    <section class="result">
      <h2>First Result</h2>
      <p>Alpha body.</p>
    </section>
    <section class="result">
      <h2>Second Result</h2>
      <p>Beta body.</p>
    </section>
    <aside><p>Sidebar body.</p></aside>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/excluded-tags",
            MockResponse::html(
                r#"
<html>
  <body>
    <main>
      <h1>Tag Filtering</h1>
      <aside>Promotional Sidebar</aside>
      <p>Kept article body.</p>
      <footer>Article Footer</footer>
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
        .route(
            "/markdown",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Guide</h1>
      <p>Intro with <strong>bold</strong> and <a href="/docs">docs</a>.</p>
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
        .route(
            "/markdown-inline-blocks",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Reference Bits</h1>
      <p>Status: <del>removed</del>, <em>soft</em>, <u>under</u>, <kbd>Cmd K</kbd>, <tt>TTY</tt>, <q>quoted</q>, <abbr title="HyperText Markup Language">HTML</abbr>.</p>
      <p>1. Not a generated list.</p>
      <p>- Not a generated bullet.</p>
      <p>+ Not a generated plus bullet.</p>
      <p>Literal \*stars\* and \[brackets\].</p>
      <hr>
      <blockquote><p>Quoted <strong>block</strong>.</p><p>Second line.</p></blockquote>
      <figure>
        <img src="/figure.png" alt="Figure alt">
        <figcaption>Figure caption with <cite>source</cite>.</figcaption>
      </figure>
      <details>
        <summary>Expandable Summary</summary>
        <p>Hidden detail text.</p>
      </details>
      <address>Contact the docs team.</address>
      <dl>
        <dt>Term</dt>
        <dd>Definition with <strong>detail</strong>.</dd>
      </dl>
    </main>
  </body>
</html>
"#,
            ),
        )
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
      <p>Read <a href="/guide" title="Guide &quot;title&quot; [v1] (draft)">the guide</a> or <a href="mailto:help@example.com">email support</a>.</p>
      <p>Canonical <a href="https://example.com/docs" title="Docs title">https://example.com/docs</a>.</p>
      <p>Jump <a href="#details">within page</a>.</p>
      <p>Empty <a href="/empty"></a> marker.</p>
      <p>Asset <a href="/release(2026)">release notes</a> and <img src="/assets/diagram(1).png" alt="A [diagram] (v1)">.</p>
      <p>Icon <a href="/download"><img src="/icons/app(1).svg" alt="Download [app]"></a>.</p>
    </main>
  </body>
</html>
"##,
            ),
        )
        .route(
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
      <img id="diagram" class="figure" src="/diagram.png" alt="Diagram" width="640" height="480" data-private="image-secret" style="display:none">
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
        .start()
}
