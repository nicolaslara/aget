use crate::support::mock_site::{MockResponse, MockSiteBuilder};

pub(crate) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    builder
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
            "/markdown-google-doc",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Google Doc</h1>
      <p><span style="font-weight:700">Bold</span> <span style="font-style:italic">Italic</span> <span style="font-family:Consolas">Code</span> <span style="text-decoration:line-through">Gone</span></p>
    </main>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/markdown-google-doc-lists",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="article">
      <h1>Google Doc Lists</h1>
      <ul style="list-style-type:decimal">
        <li style="margin-left:0px">First step</li>
        <li style="margin-left:72px">Nested step</li>
      </ul>
      <ul style="list-style-type:disc">
        <li style="margin-left:36px">Nested bullet</li>
      </ul>
    </main>
  </body>
</html>
"#,
            ),
        )
}
