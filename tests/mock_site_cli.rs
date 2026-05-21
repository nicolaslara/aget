mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::sync::OnceLock;
use std::time::Duration;

use aget::extraction::{ExtractorBackend, ExtractorBackendResult, ExtractorRequest};
use aget::{
    Aget, AgetError, ErrorCode, OutputFormat, OwnedBrowserAutomationBackend, OwnedExtractorBackend,
    Session, SessionCookie, SessionOrigin, SessionStore, StorageEntry,
};
use assert_cmd::Command;
use support::mock_site::{MockResponse, MockSite};

#[test]
fn mock_site_fetch_handles_redirect_output_shaping_and_waits() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/redirect"),
            "--content-format",
            "text",
            "--selector",
            "main",
            "--exclude-selector",
            "nav",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["final_url"], site.url("/public"));
    assert!(json["content"].as_str().unwrap().contains("Public Main"));
    assert!(!json["content"]
        .as_str()
        .unwrap()
        .contains("In-Article Navigation"));

    let delayed = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/delayed"),
            "--content-format",
            "text",
            "--wait-for-selector",
            "#ready",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let delayed_json = success_data(&delayed, "get");
    assert!(delayed_json["content"]
        .as_str()
        .unwrap()
        .contains("Delayed Ready"));
}

#[test]
fn default_cli_fetch_uses_owned_backend_without_command_dependencies() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();

    let output = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env_remove("AGET_CRAWL4AI_COMMAND")
        .env_remove("AGET_AGENT_BROWSER_COMMAND")
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/public"),
            "--content-format",
            "text",
            "--selector",
            "main",
            "--exclude-selector",
            "nav",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json = success_data(&output, "get");
    assert_eq!(json["extractor"], "aget-owned-extractor");
    assert_eq!(json["content"], "Public Main Visible public article.");
}

#[test]
fn backend_parity_covers_extractor_content_formats() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command();
    let site = MockSite::builder()
        .route(
            "/formats",
            MockResponse::html(
                r#"
<html>
  <body>
    <main>
      <h1>Format Heading</h1>
      <p>Format body text.</p>
    </main>
  </body>
</html>
"#,
            ),
        )
        .start();

    let markdown = aget(&aget_home, &fake_backend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(markdown.content_format, "markdown");
    assert!(markdown.content.contains("Format Heading"));
    assert!(markdown.content.contains("Format body text."));

    let text = aget(&aget_home, &fake_backend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(text.content_format, "text");
    assert_eq!(text.content, "Format Heading Format body text.");

    let html = aget(&aget_home, &fake_backend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert_eq!(html.content_format, "html");
    assert!(html.content.contains("<main>"));
    assert!(html.content.contains("<h1>Format Heading</h1>"));

    let json = aget(&aget_home, &fake_backend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Json)
        .exclude_selector("p.ad")
        .run()
        .unwrap();
    assert_eq!(json.content_format, "json");
    let parsed: serde_json::Value = serde_json::from_str(&json.content).unwrap();
    assert_eq!(parsed["url"], site.url("/formats"));
    assert_eq!(parsed["content"], "Format Heading Format body text.");
}

#[test]
fn homegrown_extractor_backend_covers_static_http_parity_slice() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
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
      <hr>
      <blockquote><p>Quoted <strong>block</strong>.</p><p>Second line.</p></blockquote>
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
                r#"
<html>
  <body>
    <main class="article">
      <h1>Link Defaults</h1>
      <p>Read <a href="/guide" title="Guide &quot;title&quot; [v1] (draft)">the guide</a> or <a href="mailto:help@example.com">email support</a>.</p>
      <p>Canonical <a href="https://example.com/docs">https://example.com/docs</a>.</p>
      <p>Empty <a href="/empty"></a> marker.</p>
      <p>Asset <a href="/release(2026)">release notes</a> and <img src="/assets/diagram(1).png" alt="A [diagram] (v1)">.</p>
    </main>
  </body>
</html>
"#,
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
        .start();
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let public = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/public"))
        .content_format(OutputFormat::Text)
        .selector("#content")
        .exclude_selector("nav")
        .run()
        .unwrap();
    assert_eq!(public.extractor, "aget-owned-extractor");
    assert_eq!(public.content, "Public Main Visible public article.");

    let protected = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/protected"))
        .session("app")
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert!(protected.content.contains("Protected Account"));
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));

    let text = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .selector("main.article")
        .exclude_selector("p.ad")
        .run()
        .unwrap();
    assert_eq!(text.content, "Format Heading Format body text.");

    let child_selector = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .selector("main.article > p:not(.ad)")
        .run()
        .unwrap();
    assert_eq!(child_selector.content, "Format body text.");

    let markdown = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/markdown"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    assert_eq!(
        markdown.content,
        format!(
            "# Guide\n\nIntro with **bold** and [docs]({}).\n\n* First item\n* Second `code`\n\nData Table\n\n| Name | Value |\n| --- | --- |\n| Alpha | [A\\|1]({}) |\n\n```\nlet answer = 42;\n```",
            site.url("/docs"),
            site.url("/alpha")
        )
    );

    let markdown_base = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/markdown-base"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    assert_eq!(
        markdown_base.content,
        format!(
            "# Base Links\n\nRead the [base page]({}).",
            site.url("/guide/page.html")
        )
    );

    let markdown_inline_blocks = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/markdown-inline-blocks"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    assert_eq!(
        markdown_inline_blocks.content,
        "# Reference Bits\n\nStatus: ~~removed~~, _soft_, _under_, `Cmd K`, `TTY`, \"quoted\", HTML.\n\n* * *\n\n> Quoted **block**.\n>\n> Second line.\n\nTerm\n    Definition with **detail**.\n\n  *[HTML]: HyperText Markup Language"
    );

    let markdown_nested_lists = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/markdown-nested-lists"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    assert_eq!(
        markdown_nested_lists.content,
        "# Nested Steps\n\n1. Install\n  * Open settings\n  * Confirm access\n2. Run fetch"
    );

    let markdown_links = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/markdown-links"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .run()
        .unwrap();
    let release_url = site
        .url("/release(2026)")
        .replace('(', "\\(")
        .replace(')', "\\)");
    let diagram_url = site
        .url("/assets/diagram(1).png")
        .replace('(', "\\(")
        .replace(')', "\\)");
    assert_eq!(
        markdown_links.content,
        format!(
            "# Link Defaults\n\nRead [the guide]({} \"Guide \\\"title\\\" \\[v1\\] \\(draft\\)\") or email support.\n\nCanonical <https://example.com/docs>.\n\nEmpty []({}) marker.\n\nAsset [release notes]({}) and ![A \\[diagram\\] \\(v1\\)]({}).",
            site.url("/guide"),
            site.url("/empty"),
            release_url,
            diagram_url
        )
    );

    let html = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(html.content.contains("<main class=\"article\">"));

    let cleaned_html = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(cleaned_html
        .content
        .contains("<title>Cleanup Title</title>"));
    assert!(cleaned_html.content.contains("<h1>Cleanup Main</h1>"));
    assert!(cleaned_html.content.contains("id=\"cleanup-main\""));
    assert!(cleaned_html.content.contains("class=\"article\""));
    assert!(cleaned_html.content.contains("href=\"/kept\""));
    assert!(cleaned_html.content.contains("title=\"Kept title\""));
    assert!(cleaned_html.content.contains("src=\"/diagram.png\""));
    assert!(cleaned_html.content.contains("alt=\"Diagram\""));
    assert!(cleaned_html.content.contains("width=\"640\""));
    assert!(cleaned_html.content.contains("height=\"480\""));
    assert!(cleaned_html.content.contains("id=\"inline-image\""));
    assert!(cleaned_html.content.contains("alt=\"Inline image\""));
    assert!(cleaned_html.content.contains("id=\"empty-anchor\""));
    assert!(cleaned_html.content.contains("href=\"/empty\""));
    assert!(cleaned_html.content.contains("id=\"empty-break\""));
    assert!(cleaned_html.content.contains("id=\"empty-row\""));
    assert!(cleaned_html.content.contains("id=\"empty-cell\""));
    assert!(cleaned_html.content.contains("id=\"code-space\""));
    assert!(!cleaned_html.content.contains("<meta"));
    assert!(!cleaned_html.content.contains("<link"));
    assert!(!cleaned_html.content.contains("<style"));
    assert!(!cleaned_html.content.contains("<script"));
    assert!(!cleaned_html.content.contains("<noscript"));
    assert!(!cleaned_html.content.contains("data-private"));
    assert!(!cleaned_html.content.contains("data-select"));
    assert!(!cleaned_html.content.contains("style="));
    assert!(!cleaned_html.content.contains("onclick="));
    assert!(!cleaned_html.content.contains("aria-label="));
    assert!(!cleaned_html.content.contains("rel=\"nofollow\""));
    assert!(!cleaned_html.content.contains("data:image/png;base64"));
    assert!(!cleaned_html.content.contains("QUJDRA"));
    assert!(!cleaned_html.content.contains("empty-wrapper"));
    assert!(!cleaned_html.content.contains("empty-span"));

    let selected_by_pruned_attr = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Text)
        .selector(r#"[data-select="summary"]"#)
        .run()
        .unwrap();
    assert_eq!(selected_by_pruned_attr.content, "Visible body.");

    let cleanup_markdown = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/html-cleanup"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert!(!cleanup_markdown.content.contains("data:image"));
    assert!(!cleanup_markdown.content.contains("![Inline image]"));

    let json = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Json)
        .exclude_selector("p.ad")
        .run()
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json.content).unwrap();
    assert_eq!(parsed["url"], site.url("/formats"));
    assert_eq!(parsed["content"], "Format Heading Format body text.");

    let main_text = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/main-content"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(main_text.content, "Main Story Useful body text.");

    let main_markdown = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/main-content"))
        .content_format(OutputFormat::Markdown)
        .run()
        .unwrap();
    assert_eq!(main_markdown.content, "# Main Story\n\nUseful body text.");

    let main_html = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/main-content"))
        .content_format(OutputFormat::Html)
        .run()
        .unwrap();
    assert!(main_html.content.contains("<header>Site Header</header>"));
    assert!(main_html.content.contains("<main class=\"story\">"));

    let selector_miss = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/selector-miss"))
        .content_format(OutputFormat::Text)
        .selector(".does-not-exist")
        .run()
        .unwrap();
    assert_eq!(
        selector_miss.content,
        "Selector Header Selector Main Selector body text. Selector Footer"
    );

    let excluded_tags = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/excluded-tags"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.excluded_tags", "aside,footer")
        .run()
        .unwrap();
    assert_eq!(excluded_tags.content, "Tag Filtering Kept article body.");

    let target_elements = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/excluded-tags"))
        .content_format(OutputFormat::Markdown)
        .backend_option("crawl4ai.target_elements", "h1,p")
        .run()
        .unwrap();
    assert_eq!(
        target_elements.content,
        "# Tag Filtering\n\nKept article body."
    );

    let only_text = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/markdown"))
        .content_format(OutputFormat::Markdown)
        .selector("main.article")
        .backend_option("crawl4ai.only_text", "true")
        .run()
        .unwrap();
    assert_eq!(
        only_text.content,
        format!(
            "# Guide\n\nIntro with bold and [docs]({}).\n\n* First item\n* Second code\n\nData Table\n\n| Name | Value |\n| --- | --- |\n| Alpha | [A\\|1]({}) |\n\n```\nlet answer = 42;\n```",
            site.url("/docs"),
            site.url("/alpha")
        )
    );

    let word_count_threshold = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/formats"))
        .content_format(OutputFormat::Text)
        .selector("main.article")
        .backend_option("crawl4ai.word_count_threshold", "50")
        .run()
        .unwrap();
    assert_eq!(
        word_count_threshold.content,
        "Format Heading Format body text. Promotional aside."
    );

    let invalid_word_count_threshold = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/formats"))
        .backend_option("crawl4ai.word_count_threshold", "many")
        .run()
        .unwrap_err();
    assert_eq!(
        invalid_word_count_threshold.code(),
        ErrorCode::ExtractionFailed
    );
    assert!(invalid_word_count_threshold
        .to_string()
        .contains("crawl4ai.word_count_threshold expects an integer value"));

    let invalid_timeout_option = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/formats"))
        .backend_option("crawl4ai.page_timeout", "soon")
        .run()
        .unwrap_err();
    assert_eq!(invalid_timeout_option.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_timeout_option
        .to_string()
        .contains("crawl4ai.page_timeout expects a non-negative integer number of milliseconds"));

    let unsupported_wait_until = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/formats"))
        .backend_option("crawl4ai.wait_until", "commit")
        .run()
        .unwrap_err();
    assert_eq!(unsupported_wait_until.code(), ErrorCode::ExtractionFailed);
    assert!(unsupported_wait_until.to_string().contains(
        "crawl4ai.wait_until supports only 'domcontentloaded', 'load', or 'networkidle'"
    ));

    let invalid_wait_for_images = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/formats"))
        .backend_option("crawl4ai.wait_for_images", "eventually")
        .run()
        .unwrap_err();
    assert_eq!(invalid_wait_for_images.code(), ErrorCode::ExtractionFailed);
    assert!(invalid_wait_for_images
        .to_string()
        .contains("crawl4ai.wait_for_images expects a boolean value"));

    let unsupported_option = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/formats"))
        .backend_option("crawl4ai.magic", "value")
        .run()
        .unwrap_err();
    assert_eq!(unsupported_option.code(), ErrorCode::ExtractionFailed);
    assert!(unsupported_option
        .to_string()
        .contains(
            "supported options: crawl4ai.delay_before_return_html, crawl4ai.excluded_tags, crawl4ai.only_text, crawl4ai.page_timeout, crawl4ai.target_elements, crawl4ai.wait_for_images, crawl4ai.wait_for_timeout, crawl4ai.wait_until, crawl4ai.word_count_threshold"
        ));

    let redirect = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/redirect"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(redirect.final_url, site.url("/public"));

    let waited = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/wait-ready"))
        .content_format(OutputFormat::Text)
        .wait_for_selector("body main #ready")
        .run()
        .unwrap();
    assert_eq!(waited.content, "Ready Now");

    let js_wait = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/wait-ready"))
        .wait_for_selector("js:() => true")
        .run()
        .unwrap_err();
    assert_eq!(js_wait.code(), ErrorCode::ExtractionFailed);
}

#[test]
fn owned_browser_fallback_replays_cookie_backed_session_without_agent_browser() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let fallback = Aget::new(&aget_home)
        .with_extractor_backend(FailingExtractor)
        .with_browser_automation_backend(OwnedBrowserAutomationBackend)
        .get(site.url("/protected"))
        .session("app")
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();

    assert_eq!(fallback.extractor, "aget-owned-browser-fallback");
    assert_eq!(fallback.content, "Protected Account Private account body.");
    assert_eq!(
        fallback.warnings,
        vec!["aget-owned fallback used after primary extractor failed"]
    );
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn owned_browser_fallback_renders_cookie_backed_scripted_page_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/client-rendered",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Client Shell</h1>
      <div id="client-result">Loading</div>
    </main>
    <script>
      setTimeout(() => {
        document.querySelector("#client-result").innerHTML = "<p>Client Rendered</p>"
      }, 25)
    </script>
  </body>
</html>
"##,
            ),
        )
        .start();
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let fallback = Aget::new(&aget_home)
        .with_extractor_backend(FailingExtractor)
        .with_browser_automation_backend(OwnedBrowserAutomationBackend)
        .get(site.url("/client-rendered"))
        .session("app")
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();

    assert_eq!(fallback.extractor, "aget-owned-browser-fallback");
    assert_eq!(fallback.content, "Client Shell Client Rendered");
    assert!(site.received_cookie("/client-rendered", "app_session", "valid-app"));
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn owned_browser_fallback_renders_local_storage_backed_session_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = storage_rendered_site();
    save_storage_session(
        &aget_home,
        "storage",
        &site.origin(),
        "local_token",
        "storage-secret",
    );

    let fallback = Aget::new(&aget_home)
        .with_extractor_backend(FailingExtractor)
        .with_browser_automation_backend(OwnedBrowserAutomationBackend)
        .get(site.url("/storage-rendered"))
        .session("storage")
        .content_format(OutputFormat::Text)
        .wait_for_selector("#storage-result h1")
        .run()
        .unwrap();

    assert_eq!(fallback.extractor, "aget-owned-browser-fallback");
    assert_eq!(fallback.content, "Storage App Shell Storage Protected");
    assert!(site.received_header("/storage-api", "x-local-token", "storage-secret"));
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn owned_extractor_backend_renders_local_storage_backed_session_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = storage_rendered_site();
    save_storage_session(
        &aget_home,
        "storage",
        &site.origin(),
        "local_token",
        "storage-secret",
    );

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .with_browser_automation_backend(OwnedBrowserAutomationBackend)
        .get(site.url("/storage-rendered"))
        .session("storage")
        .content_format(OutputFormat::Text)
        .wait_for_selector("#storage-result h1")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Storage App Shell Storage Protected");
    assert!(site.received_header("/storage-api", "x-local-token", "storage-secret"));
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn owned_extractor_backend_renders_waited_javascript_page_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/delayed"))
        .content_format(OutputFormat::Text)
        .wait_for_selector("#ready")
        .backend_option("crawl4ai.page_timeout", "5000")
        .backend_option("crawl4ai.wait_for_timeout", "1000")
        .backend_option("crawl4ai.wait_until", "domcontentloaded")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert!(extraction.content.contains("Delayed Ready"));
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn owned_extractor_backend_renders_scripted_page_without_wait_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/client-rendered",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Client Shell</h1>
      <div id="client-result">Loading</div>
    </main>
    <script>
      setTimeout(() => {
        document.querySelector("#client-result").innerHTML = "<p>Client Rendered</p>"
      }, 25)
    </script>
  </body>
</html>
"##,
            ),
        )
        .start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/client-rendered"))
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Client Shell Client Rendered");
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn owned_extractor_backend_honors_render_delay_option_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/slow-client-rendered",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Slow Client Shell</h1>
      <div id="client-result">Loading</div>
    </main>
    <script>
      setTimeout(() => {
        document.querySelector("#client-result").innerHTML = "<p>Slow Client Rendered</p>"
      }, 200)
    </script>
  </body>
</html>
"##,
            ),
        )
        .start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/slow-client-rendered"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.delay_before_return_html", "0.4")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Slow Client Shell Slow Client Rendered");
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn owned_extractor_backend_honors_wait_for_images_option_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/image-wait",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Image Shell</h1>
      <p id="image-status">Image Pending</p>
      <img id="slow-image" src="/slow-image.svg" alt="slow">
    </main>
    <script>
      const status = document.querySelector("#image-status")
      const image = document.querySelector("#slow-image")
      image.addEventListener("load", () => { status.textContent = "Image Loaded" })
      image.addEventListener("error", () => { status.textContent = "Image Failed" })
    </script>
  </body>
</html>
"##,
            ),
        )
        .route(
            "/slow-image.svg",
            MockResponse::html(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"></svg>"#,
            )
            .content_type("image/svg+xml")
            .delay(Duration::from_millis(250)),
        )
        .start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/image-wait"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.wait_until", "domcontentloaded")
        .backend_option("crawl4ai.delay_before_return_html", "0")
        .backend_option("crawl4ai.wait_for_images", "true")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Image Shell Image Loaded");
    assert!(extraction.warnings.is_empty());
}

#[test]
#[ignore = "requires local Chrome/Chromium; set AGET_CHROME_COMMAND if auto-discovery fails"]
fn owned_extractor_backend_honors_networkidle_wait_until_with_chrome() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::builder()
        .route(
            "/networkidle",
            MockResponse::html(
                r##"
<html>
  <body>
    <main>
      <h1>Network Shell</h1>
      <p id="network-result">Loading</p>
    </main>
    <script>
      fetch("/slow-fragment")
        .then((response) => response.text())
        .then((text) => { document.querySelector("#network-result").textContent = text })
    </script>
  </body>
</html>
"##,
            ),
        )
        .route(
            "/slow-fragment",
            MockResponse::html("Network Settled")
                .content_type("text/plain")
                .delay(Duration::from_millis(650)),
        )
        .start();

    let extraction = Aget::new(&aget_home)
        .with_extractor_backend(OwnedExtractorBackend)
        .get(site.url("/networkidle"))
        .content_format(OutputFormat::Text)
        .backend_option("crawl4ai.wait_until", "networkidle")
        .backend_option("crawl4ai.delay_before_return_html", "0")
        .backend_option("crawl4ai.page_timeout", "5000")
        .run()
        .unwrap();

    assert_eq!(extraction.extractor, "aget-owned-extractor");
    assert_eq!(extraction.content, "Network Shell Network Settled");
}

#[test]
fn mock_site_replays_cookie_and_storage_sessions() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");
    save_storage_session(
        &aget_home,
        "storage",
        &site.origin(),
        "local_token",
        "storage-secret",
    );

    let protected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--session",
            "app",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let protected_json = success_data(&protected, "get");
    assert!(protected_json["content"]
        .as_str()
        .unwrap()
        .contains("Protected Account"));
    assert_eq!(protected_json["sensitive"], true);
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));

    let storage = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/storage-protected"),
            "--session",
            "storage",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let storage_json = success_data(&storage, "get");
    assert!(storage_json["content"]
        .as_str()
        .unwrap()
        .contains("Storage Protected"));
    assert!(site.received_header("/storage-api", "x-local-token", "storage-secret"));
}

#[test]
fn mock_site_covers_unauthenticated_expired_and_logout_states() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    save_cookie_session(
        &aget_home,
        "expired",
        &site.host(),
        "app_session",
        "expired",
    );
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let unauthenticated = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let unauthenticated_json = success_data(&unauthenticated, "get");
    assert_eq!(
        unauthenticated_json["final_url"],
        site.url("/login?next=/protected")
    );
    assert!(unauthenticated_json["content"]
        .as_str()
        .unwrap()
        .contains("Login Form"));
    assert!(!site.received_cookie("/protected", "app_session", "valid-app"));

    let expired = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--session",
            "expired",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let expired_json = success_data(&expired, "get");
    assert!(expired_json["content"]
        .as_str()
        .unwrap()
        .contains("Session Expired"));

    let logout = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/logout"),
            "--session",
            "app",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let logout_json = success_data(&logout, "get");
    assert!(logout_json["content"]
        .as_str()
        .unwrap()
        .contains("Logged Out"));
}

#[test]
fn mock_site_imported_chrome_session_can_fetch_protected_page() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    let fake_agent_browser = mock_agent_browser_command();

    let imported = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--envelope",
            "json",
            "session",
            "import",
            "chrome",
            "--chrome-profile",
            "Default",
            "--name",
            "imported",
            "--allow-domain",
            &site.host(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let import_json = success_data(&imported, "session.import.chrome");
    assert_eq!(import_json["name"], "imported");
    assert_eq!(import_json["cookie_count"], 1);

    let protected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--session",
            "imported",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let protected_json = success_data(&protected, "get");
    assert!(protected_json["content"]
        .as_str()
        .unwrap()
        .contains("Protected Account"));
    assert!(site.received_cookie("/protected", "app_session", "valid-app"));
}

#[test]
fn documents_session_compose_replay_and_scope_rejection_contract() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    save_cookie_session(
        &aget_home,
        "provider",
        &site.host(),
        "provider_session",
        "valid-provider",
    );
    save_cookie_session(&aget_home, "app", &site.host(), "app_session", "valid-app");

    let compose = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args([
            "--envelope",
            "json",
            "session",
            "compose",
            "combined",
            "--session",
            "provider",
            "--session",
            "app",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let compose_json = success_data(&compose, "session.compose");
    assert_eq!(compose_json["cookie_count"], 2);

    let composed_fetch = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/requires-two"),
            "--session",
            "combined",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let composed_json = success_data(&composed_fetch, "get");
    assert!(composed_json["content"]
        .as_str()
        .unwrap()
        .contains("Composed Session"));
    assert!(site.received_cookie("/requires-two", "provider_session", "valid-provider"));
    assert!(site.received_cookie("/requires-two", "app_session", "valid-app"));

    save_mixed_scope_session(&aget_home, "mixed", &site.host());
    let requests_before_rejection = site.requests().len();
    let rejected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--session",
            "mixed",
        ])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let rejected_json: serde_json::Value = serde_json::from_slice(&rejected).unwrap();
    assert_eq!(rejected_json["command"], "get");
    assert_eq!(rejected_json["error"]["code"], "privacy_policy_blocked");
    assert_eq!(site.requests().len(), requests_before_rejection);
}

#[test]
fn mock_site_login_bootstrap_can_fetch_protected_page_without_manual_action() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    let fake_agent_browser = mock_agent_browser_command();

    Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "mock-app",
            "--url",
            &site.https_url("/login"),
        ])
        .assert()
        .success();

    let finish = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "finish",
            "mock-app",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let finish_json = success_data(&finish, "session.login.finish");
    assert_eq!(finish_json["name"], "mock-app");

    let protected = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_CRAWL4AI_COMMAND", &fake_backend)
        .args([
            "--envelope",
            "json",
            "get",
            "--inline-content",
            "always",
            &site.url("/protected"),
            "--session",
            "mock-app",
            "--content-format",
            "text",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let protected_json = success_data(&protected, "get");
    assert!(protected_json["content"]
        .as_str()
        .unwrap()
        .contains("Protected Account"));
}

#[test]
fn documents_public_get_json_contract_for_agents() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();

    let result = aget(&aget_home, &fake_backend)
        .get(site.url("/public"))
        .content_format(OutputFormat::Text)
        .selector("main")
        .exclude_selector("nav")
        .run()
        .unwrap();

    assert_eq!(result.url, site.url("/public"));
    assert_eq!(result.final_url, site.url("/public"));
    assert_eq!(result.content_format, "text");
    assert_eq!(result.extractor, "crawl4ai");
    assert_eq!(result.content, "Public Main Visible public article.");
    assert!(result.sessions.is_empty());
    assert!(!result.sensitive);
    assert!(result.warnings.is_empty());
    assert!(result.timing_ms.total > 0);
    assert_eq!(result.limits.max_chars, None);
    assert!(!result.limits.truncated);
    assert_eq!(result.output_options.content_format, OutputFormat::Text);
    assert_eq!(result.output_options.selector.as_deref(), Some("main"));
    assert_eq!(
        result.output_options.exclude_selector.as_deref(),
        Some("nav")
    );
    assert!(result.output_options.backend_options.is_empty());

    let content_path = PathBuf::from(&result.artifacts.content);
    let metadata_path = PathBuf::from(&result.artifacts.metadata);
    assert!(content_path.starts_with(aget_home.join("runs")));
    assert!(metadata_path.starts_with(aget_home.join("runs")));
    assert_eq!(
        fs::read_to_string(content_path).unwrap(),
        "Public Main Visible public article.\n"
    );

    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
    assert_eq!(metadata["url"], site.url("/public"));
    assert_eq!(metadata["content_format"], "text");
    assert_eq!(metadata["sensitive"], false);
}

#[test]
fn documents_output_limits_out_file_and_warning_contract() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    let out_path = temp.path().join("agent-context.txt");

    let result = aget(&aget_home, &fake_backend)
        .get(site.url("/warning"))
        .content_format(OutputFormat::Text)
        .output(&out_path)
        .max_chars(12)
        .run()
        .unwrap();

    assert_eq!(result.warnings, vec!["mock warning"]);
    assert_eq!(result.content, "Warning Page");
    assert_eq!(result.artifacts.content, out_path.to_string_lossy());
    assert_eq!(fs::read_to_string(&out_path).unwrap(), "Warning Page\n");
    assert_eq!(result.limits.max_chars, Some(12));
    assert!(result.limits.truncated);
    assert_eq!(result.limits.truncated_by.as_deref(), Some("max_chars"));
    assert!(result.limits.content_chars_before_truncation > 12);
    assert_eq!(result.limits.content_chars_after_truncation, 12);
}

#[test]
fn documents_custom_site_routes_for_extraction_features() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let fake_backend = mock_backend_command();
    let site = MockSite::builder()
        .route(
            "/guide",
            MockResponse::html(
                r#"
<html>
  <body>
    <main>
      <h1>Custom Guide</h1>
      <p>Feature-specific extraction fixture.</p>
      <aside>Remove this sidebar</aside>
    </main>
  </body>
</html>
"#,
            )
            .header("X-Fixture", "custom-guide"),
        )
        .route("/guide/latest", MockResponse::redirect("/guide"))
        .start();

    let result = aget(&aget_home, &fake_backend)
        .get(site.url("/guide/latest"))
        .content_format(OutputFormat::Text)
        .selector("main")
        .run()
        .unwrap();

    assert_eq!(result.final_url, site.url("/guide"));
    assert_eq!(
        result.content,
        "Custom Guide Feature-specific extraction fixture. Remove this sidebar"
    );
}

#[test]
fn documents_session_lifecycle_contract_for_agents() {
    let temp = tempfile::tempdir().unwrap();
    let aget_home = temp.path().join("aget-home");
    let site = MockSite::start();
    let fake_backend = mock_backend_command();
    let fake_agent_browser = mock_agent_browser_command();

    let start = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "start",
            "mock-app",
            "--url",
            &site.https_url("/login"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_data = success_data(&start, "session.login.start");
    assert_eq!(start_data["state"], "login_started");
    assert_eq!(start_data["name"], "mock-app");
    assert_eq!(
        start_data["allowed_domains"],
        serde_json::json!([site.host()])
    );
    assert_eq!(
        start_data["next_command"],
        serde_json::json!(["aget", "session", "login", "finish", "mock-app"])
    );

    let finish = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .env("AGET_AGENT_BROWSER_COMMAND", &fake_agent_browser)
        .args([
            "--envelope",
            "json",
            "session",
            "login",
            "finish",
            "mock-app",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let finish_data = success_data(&finish, "session.login.finish");
    assert_eq!(finish_data["state"], "login_finished");
    assert_eq!(finish_data["source"], "agent_browser");
    assert_eq!(finish_data["cookie_count"], 1);

    let list = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "session", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list_data = success_data(&list, "session.list");
    assert_eq!(list_data["sessions"], serde_json::json!(["mock-app"]));

    let inspect = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "session", "inspect", "mock-app"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect_data = success_data(&inspect, "session.inspect");
    assert_eq!(inspect_data["name"], "mock-app");
    assert_eq!(inspect_data["sensitive"], true);
    assert_eq!(inspect_data["cookies"][0]["name"], "app_session");
    assert_eq!(inspect_data["cookies"][0]["value"], "<redacted>");

    let fetch = aget(&aget_home, &fake_backend)
        .get(site.url("/protected"))
        .session("mock-app")
        .content_format(OutputFormat::Text)
        .run()
        .unwrap();
    assert_eq!(fetch.sessions, vec!["mock-app"]);
    assert!(fetch.sensitive);
    assert!(fetch.content.contains("Protected Account"));

    let delete = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "session", "delete", "mock-app"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let delete_data = success_data(&delete, "session.delete");
    assert_eq!(delete_data["deleted"], true);

    let post_delete_list = Command::cargo_bin("aget")
        .unwrap()
        .env("AGET_HOME", &aget_home)
        .args(["--envelope", "json", "session", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let post_delete_data = success_data(&post_delete_list, "session.list");
    assert_eq!(post_delete_data["sessions"], serde_json::json!([]));

    let post_delete_fetch = aget(&aget_home, &fake_backend)
        .get(site.url("/protected"))
        .session("mock-app")
        .run()
        .unwrap_err();
    assert_eq!(post_delete_fetch.code(), aget::ErrorCode::IoError);
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn mock_backend_command() -> String {
    shell_quote(&mock_tool_path("aget-mock-backend").to_string_lossy())
}

fn mock_agent_browser_command() -> PathBuf {
    mock_tool_path("aget-mock-agent-browser")
}

fn mock_tool_path(name: &str) -> PathBuf {
    let tools = MOCK_TOOLS.get_or_init(build_mock_tools);
    let binary = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    tools.target_dir.join("debug").join(binary)
}

struct MockTools {
    target_dir: PathBuf,
}

static MOCK_TOOLS: OnceLock<MockTools> = OnceLock::new();

fn build_mock_tools() -> MockTools {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = root.join("tests/fixtures/mock-tools/Cargo.toml");
    let target_dir = root.join("target/aget-mock-tools");
    let status = StdCommand::new("cargo")
        .args([
            "build",
            "--quiet",
            "--manifest-path",
            manifest.to_str().unwrap(),
            "--target-dir",
            target_dir.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "failed to build mocked e2e helper tools");
    MockTools { target_dir }
}

fn aget(home: &Path, backend: &str) -> Aget {
    Aget::new(home).with_backend_command(backend.to_string())
}

fn success_data(output: &[u8], command: &str) -> serde_json::Value {
    let json: serde_json::Value = serde_json::from_slice(output).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["command"], command);
    json["data"].clone()
}

fn storage_rendered_site() -> MockSite {
    MockSite::builder()
        .route(
            "/storage-rendered",
            MockResponse::html(
                r##"
<html>
  <body>
    <main><h1>Storage App Shell</h1><div id="storage-result"></div></main>
    <script>
      fetch("/storage-api", { headers: { "X-Local-Token": localStorage.getItem("local_token") }})
        .then((response) => response.text())
        .then((html) => { document.querySelector("#storage-result").innerHTML = html })
    </script>
  </body>
</html>
"##,
            ),
        )
        .start()
}

fn save_cookie_session(home: &Path, name: &str, domain: &str, cookie_name: &str, value: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_cookie_domains.push(domain.to_string());
    session.cookies.push(SessionCookie {
        name: cookie_name.to_string(),
        value: value.to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}

fn save_storage_session(home: &Path, name: &str, origin: &str, key: &str, value: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_storage_origins.push(origin.to_string());
    session.origins.push(SessionOrigin {
        origin: origin.to_string(),
        local_storage: vec![StorageEntry {
            name: key.to_string(),
            value: value.to_string(),
        }],
        session_storage: Vec::new(),
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}

fn save_mixed_scope_session(home: &Path, name: &str, domain: &str) {
    let store = SessionStore::new(home).unwrap();
    let mut session = Session::new(name);
    session.allowed_cookie_domains.push(domain.to_string());
    session.cookies.push(SessionCookie {
        name: "app_session".to_string(),
        value: "valid-app".to_string(),
        domain: domain.to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    session.cookies.push(SessionCookie {
        name: "other_session".to_string(),
        value: "other-secret".to_string(),
        domain: "unrelated.example".to_string(),
        path: "/".to_string(),
        expires: None,
        http_only: true,
        secure: false,
        same_site: Some("Lax".to_string()),
        source_session: Some(name.to_string()),
    });
    store.save(&session).unwrap();
}

#[derive(Clone)]
struct FailingExtractor;

impl ExtractorBackend for FailingExtractor {
    fn name(&self) -> &'static str {
        "failing-extractor"
    }

    fn extract(&self, _request: ExtractorRequest<'_>) -> Result<ExtractorBackendResult, AgetError> {
        Err(AgetError::Stable {
            code: ErrorCode::ExtractionFailed,
            message: "primary extractor failed".to_string(),
        })
    }
}
