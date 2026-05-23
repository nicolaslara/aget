use crate::support::mock_site::{MockResponse, MockSiteBuilder};

pub(super) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    builder
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
            "/consent-popup-content",
            MockResponse::html(
                r#"
<html>
  <body>
    <main class="story">
      <h1>Consent Story</h1>
      <div id="cookie-notice">Consent notice text.</div>
      <section role="dialog">Newsletter modal text.</section>
      <p>Useful consent article text.</p>
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
}
