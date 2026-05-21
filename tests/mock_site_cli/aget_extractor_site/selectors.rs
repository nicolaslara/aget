use crate::support::mock_site::{MockResponse, MockSiteBuilder};

pub(crate) fn routes(builder: MockSiteBuilder) -> MockSiteBuilder {
    builder
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
            "/remove-forms",
            MockResponse::html(
                r#"
<html>
  <body>
    <main>
      <h1>Form Cleanup</h1>
      <form>
        <label>Private form label</label>
        <input name="secret" value="do-not-keep">
      </form>
      <p>Kept content.</p>
    </main>
  </body>
</html>
"#,
            ),
        )
}
