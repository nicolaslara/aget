use crate::support::mock_site::{MockResponse, MockSiteBuilder};

pub(super) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    builder
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
