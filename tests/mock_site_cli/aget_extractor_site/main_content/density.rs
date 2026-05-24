use crate::support::mock_site::{MockResponse, MockSiteBuilder};

pub(super) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    builder
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
      <p>The real article should win because generic pruning ignores candidates inside page chrome.</p>
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
}
