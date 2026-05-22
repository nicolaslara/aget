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
        .route(
            "/markdown-inline-blocks",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Reference Bits</h1>
      <p>Status: <del>removed</del>, <em>soft</em>, <u>under</u>, <kbd>Cmd K</kbd>, <tt>TTY</tt>, <q>quoted</q>, <abbr title="HyperText Markup Language">HTML</abbr>, power <sup>2</sup>, and water <sub>2</sub>.</p>
      <p><span style="text-decoration: line-through">Styled removed text.</span></p>
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
            "/markdown-escape-snob",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Escape Snob</h1>
      <p>Escapes `tick`, *star*, _under_, {brace}, [bracket], (paren), #hash, and bang!.</p>
      <p><code>*code*</code> stays code.</p>
      <pre><code># raw *code*</code></pre>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/markdown-unicode-snob",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Unicode Snob</h1>
      <p>&copy; &#8212; &ldquo;quote&rdquo; &rarr; &larr; &middot; &oelig;uvre caf&eacute;.</p>
    </main>
  </body>
</html>
"#,
            ),
        )
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
