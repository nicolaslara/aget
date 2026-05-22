use crate::support::mock_site::{MockResponse, MockSiteBuilder};

pub(crate) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
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
        .route(
            "/main-content-page-chrome",
            MockResponse::html(
                r#"
<html>
  <body>
    <header>
      <section id="content" class="story-body">
        <h1>Header Content Teaser</h1>
        <p>This header block uses content labels and many words that could otherwise look like the article body.</p>
        <p>Navigation chrome should not win default extraction even when it looks content-heavy.</p>
      </section>
    </header>
    <article class="story">
      <h1>Real Article</h1>
      <p>The real article should win because Crawl4AI-style pruning ignores candidates inside page chrome.</p>
    </article>
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
            "/main-content-unlabeled-density",
            MockResponse::html(
                r#"
<html>
  <body>
    <nav>
      <a href="/docs">Docs</a>
      <a href="/pricing">Pricing</a>
      <a href="/community">Community</a>
    </nav>
    <div>
      <h1>Unlabeled Report</h1>
      <p>This unlabeled report carries the main body text even though it has no helpful class, id, role, or aria label.</p>
      <p>The content block should win because it has strong prose density and very few links.</p>
    </div>
    <footer>Footer links and legal navigation</footer>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/main-content-body-fallback-chrome",
            MockResponse::html(
                r#"
<html>
  <body>
    <header>Global Header</header>
    <nav>
      <a href="/docs">Docs</a>
      <a href="/pricing">Pricing</a>
    </nav>
    <h1>Bare Body Story</h1>
    <p>The useful article paragraph has no wrapper, so body fallback should keep it without page chrome.</p>
    <footer>Footer links and legal navigation</footer>
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
            "/main-content-embedded-noise-label",
            MockResponse::html(
                r#"
<html>
  <body>
    <article class="content story article-comments-panel">
      <h1>Long Discussion</h1>
      <p>This discussion thread is intentionally long and repetitive so a scoring-only heuristic could prefer it over the actual source article.</p>
      <p>Replies include migration notes, setup examples, release commentary, troubleshooting details, and documentation references that look useful but are not the main page content.</p>
      <p>Additional copied discussion text keeps adding words to make the noisy candidate deceptively strong for default extraction.</p>
    </article>
    <div id="content" class="story-body">
      <h1>Source Article</h1>
      <p>The source article should win because embedded comments labels exclude noisy candidates.</p>
    </div>
  </body>
</html>
"#,
            ),
        )
        .route(
            "/main-content-word-threshold",
            MockResponse::html(
                r#"
<html>
  <body>
    <article class="story">
      <h1>Short Teaser</h1>
      <p>Brief story wins normally.</p>
    </article>
    <div id="content" class="content">
      <h1>Long Article</h1>
      <p>This fallback article has enough useful words to pass the configured threshold.</p>
    </div>
  </body>
</html>
"#,
            ),
        )
}
