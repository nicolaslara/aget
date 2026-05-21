### D55: I19d starts with an owned static extractor after Crawl4AI source inspection

I19d inspected the local Crawl4AI snapshot before porting extraction behavior. Relevant source paths:

- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py`: `AsyncWebCrawler.arun` composes the high-level fetch pipeline: cache/robots checks, crawler strategy navigation, response handling, HTML processing, markdown/content selection, and result metadata.
- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`: the Playwright strategy owns browser/page/context acquisition, navigation, and post-load DOM content retrieval.
- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `BrowserConfig` and run configuration show that `aget`'s current adapter uses only a narrow subset: headless Chromium, storage-state input, viewport/channel, CSS waits, selectors/exclusions, and cache bypass.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: scraping/content cleanup is a separable stage after rendered HTML is available.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` and `crawl4ai/html2text/`: markdown generation is a distinct HTML-to-markdown layer after cleanup.

The porting map for `aget` should keep those layers separate:

1. Transport/render layer: fetch a URL with explicit session state.
2. HTML processing layer: select/exclude/wait against page HTML.
3. Content conversion layer: produce markdown/html/text/json.
4. Finalization layer: artifacts, warnings, final URL, truncation, and stable errors remain in `src/extraction.rs`.

Do not port Crawl4AI's anti-bot/proxy/stealth paths, arbitrary JavaScript waits, LLM extraction, cache policy surface, or broad crawler features into this migration slice. Those are either outside the current `aget` dependency contract or conflict with the authorization-only safety boundary.

First implementation slice:

- Added `OwnedExtractorBackend` behind `ExtractorBackend`.
- It is not the default runtime backend yet; I19f owns the default switch.
- It initially used no new third-party dependencies and copied no upstream Crawl4AI code. The later D56 slice adds Rust dependencies for transport and parsing while preserving behavior-driven porting.
- It supports deterministic local/static HTTP extraction: redirects, cookie replay from structured `PlaywrightState`, text/html/json/markdown-as-text output, simple tag/id/class/tag.class selectors, comma-separated simple exclusions, CSS-only wait validation, artifact writes, and owned-backend error metadata.
- It deliberately does not claim browser-rendered parity yet: HTTPS/TLS, JavaScript rendering, localStorage replay through page scripts, richer CSS selectors, Crawl4AI-quality markdown/readability, screenshots, and real browser timeouts remain open for the next I19d/I19e slices.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli backend_parity_covers_extractor_content_formats`
- `cargo test --test aget_api`
- `cargo test --test get_cli get_real_helper_rejects_javascript_wait_before_crawl4ai_import`

Confidence: Medium. The first owned backend slice is narrow, local-only, and tested without Crawl4AI or `agent-browser`, but I19d remains in progress because browser-rendered extraction and localStorage-backed authenticated replay are not yet owned.

### D56: I19d replaces ad hoc owned extractor internals with Rust transport and CSS parsing crates

The second owned-extractor slice replaces the first slice's hand-rolled HTTP/selector implementation with focused Rust crates while keeping the same `ExtractorBackend` boundary:

- `ureq` v3.3.0 for blocking HTTP(S), redirects, and global request timeouts. License: MIT OR Apache-2.0.
- `scraper` v0.27.0 for HTML5 parsing and CSS selector matching. License: ISC.
- `html5ever` v0.39.0 as a direct dependency only for the `TreeSink` trait needed to detach excluded nodes from `scraper`'s parsed tree. License: MIT OR Apache-2.0.

The owned extractor now supports HTTPS-capable transport at the Rust layer and richer CSS selectors than the first static slice, including descendant/child/not-class selectors that are relevant to current `--selector`, `--exclude-selector`, and CSS-only `--wait-for-selector` behavior. It still does not execute page JavaScript and therefore does not replace Crawl4AI's browser-rendered SPA behavior yet.

Safety notes:

- Cookie replay remains explicit and scoped through `PlaywrightState`; secure cookies are only sent to HTTPS URLs.
- JavaScript wait strings are still rejected before selector parsing.
- Backend-specific `crawl4ai.*` options remain unsupported by the owned extractor until an `aget`-owned option namespace is designed; command-backed compatibility remains available.
- No Crawl4AI source or tests were copied in this slice. Upstream Crawl4AI was used only for architecture/source inspection recorded in D55.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high for this slice. It closes the ad hoc selector/transport gap in the static owned extractor, but I19d remains open for JavaScript-rendered extraction, localStorage replay through page scripts, and markdown/readability quality.

### D57: I19d adds a first owned HTML-to-markdown slice

Before porting markdown behavior, I19d inspected Crawl4AI's markdown and cleaned-HTML flow:

- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py` selects the HTML source for markdown generation (`cleaned_html`, `raw_html`, or `fit_html`) after scraping/content processing.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` uses `DefaultMarkdownGenerator` plus `CustomHTML2Text`, then optionally converts links to citations and produces filtered/fit markdown.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py` removes excluded tags/selectors before cleaned HTML reaches markdown generation.
- `references/repos/crawl4ai/tests/async/test_markdown_genertor.py` and `tests/regression/test_reg_content.py` cover links/citations, content filters, and selector/exclusion behavior at a higher quality bar than the first `aget`-owned slice.

The Rust slice keeps the same layer boundary without copying Crawl4AI code. `OwnedExtractorBackend` now renders `OutputFormat::Markdown` through a small in-process DOM renderer instead of aliasing markdown to normalized text. The renderer currently handles the static/documentation structures `aget` tests directly: headings, paragraphs, emphasis, links/images, unordered/ordered lists, inline code, fenced code blocks, and blockquotes. Text output is unchanged and still uses normalized text.

License and dependency notes:

- The `html2md` Rust crate was rejected for this project because `cargo info html2md` reports GPL-3.0+.
- `ego-tree` v0.11.0 is now a direct dependency so the renderer can traverse the `scraper` DOM explicitly. License: ISC.
- No Crawl4AI source or tests were copied. The local Crawl4AI snapshot was used only to identify source-layer behavior and quality targets.

Remaining markdown/readability gaps are deliberate follow-ups: Crawl4AI-style citations/references, GFM tables, cleaned-main-content/readability pruning, fit markdown, media/link metadata, and broader edge-case parity. Browser-rendered JavaScript and localStorage-backed replay are still separate I19d/I19e gaps.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium for this slice. It replaces the most obvious markdown-as-text gap with tested structural markdown, but it is not yet a full Crawl4AI-quality markdown/readability replacement.

### D58: I19e starts with owned session-backed fallback extraction

Before porting the first `agent-browser` behavior, I19e inspected the local `agent-browser` snapshot paths that implement the current `aget get` fallback shape:

- `references/repos/agent-browser/cli/src/commands.rs`: parses `open`, `state load`, `get html body`, `get text body`, and `close` command shapes.
- `references/repos/agent-browser/cli/src/native/state.rs`: loads Playwright-style storage state by setting cookies and navigating to storage origins before setting local/session storage through CDP.
- `references/repos/agent-browser/cli/src/native/browser.rs`: navigates with CDP, tracks final page URL/title, and extracts DOM content through runtime evaluation.
- `references/repos/agent-browser/cli/src/native/actions.rs` and `native/element.rs`: implement `get html <selector>` as selected element `innerHTML` and `get text <selector>` as selected element text.

The current command fallback in `aget` uses this narrow sequence only after a session-backed primary extraction failure: load composed state into a temporary browser profile, open the URL, read body HTML/text, close the browser session, and clean up temp profile state. The first owned I19e slice therefore adds `OwnedBrowserAutomationBackend` with an owned fallback extraction path for static cookie-backed pages. It reuses the I19d owned fetch/HTML processing pipeline, uses structured `PlaywrightState` directly rather than an `agent-browser` state file, defaults fallback extraction to `body` to match the command fallback shape, and returns extractor metadata as `aget-owned-browser-fallback`.

This is intentionally not the full `agent-browser` replacement. `OwnedBrowserAutomationBackend` returns explicit unsupported-capability errors for Chrome/profile import and login start/finish/cancel until a real CDP/profile implementation is ported. It also does not execute page JavaScript or apply localStorage through a browser context, so localStorage-backed fallback pages still depend on future CDP/browser work.

Validation:

- `cargo test --test mock_site_cli owned_browser_fallback_replays_cookie_backed_session_without_agent_browser`

Confidence: Medium. The slice removes an `agent-browser` dependency path for static cookie-backed fallback extraction and is covered by deterministic MockSite evidence, but the high-risk browser automation work remains open.

### D59: I19d resolves owned markdown links against the page base URL

Before porting this behavior, I19d inspected Crawl4AI's link handling in `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`. Crawl4AI's `DefaultMarkdownGenerator` passes a base URL into its HTML-to-markdown converter and resolves relative markdown links before building citation references. Its tests in `references/repos/crawl4ai/tests/async/test_markdown_genertor.py` cover relative links and image URLs against a supplied base URL.

The owned renderer now resolves link and image URLs with the Rust `url` crate. It uses the final fetched URL as the default markdown base and honors an HTML `<base href="...">` element before extraction, matching Crawl4AI's separation between cleaned HTML and markdown generation. This keeps markdown output agent-ready when a selected content block contains relative links.

License and dependency notes:

- `url` v2.5.8 is now a direct dependency for standards-based URL joining. License: MIT OR Apache-2.0.
- No Crawl4AI source or tests were copied; the deterministic MockSite route was authored in this repo from the observed behavior target.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high for this slice. It closes a concrete markdown parity gap with primary-source behavior inspection and local deterministic coverage; citation/reference formatting and broader readability quality remain open.

### D60: I19e adds a minimal owned CDP renderer for localStorage-backed fallback

Before porting this behavior, I19e inspected `agent-browser`'s storage-state and CDP paths again:

- `references/repos/agent-browser/cli/src/native/state.rs` loads Playwright-style cookies first, then navigates to each storage origin and sets `localStorage`/`sessionStorage` through `Runtime.evaluate` before the target page is opened.
- `references/repos/agent-browser/cli/src/native/element.rs` extracts element HTML through CDP after resolving the selector.
- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` launches Chrome with a temporary user data directory, waits for `DevToolsActivePort`, and cleans up the temp profile after shutdown.

`OwnedBrowserAutomationBackend` now keeps the fast static fallback for cookie-only sessions but switches to a minimal owned Chrome/CDP renderer when composed state contains localStorage origins. The CDP path launches a temporary local Chrome profile, attaches to a page target, sets cookies with `Network.setCookies`, navigates each localStorage origin to set storage, opens the requested URL, optionally waits for a CSS selector through an internally generated `document.querySelector(...)` expression, then feeds the rendered document HTML back into the owned extraction/formatting pipeline. This preserves the safety rule that user-provided JavaScript waits are not executed; user input is still limited to CSS selectors and is JSON-quoted inside agent-owned CDP expressions. Owned Chrome profile temp dirs are removed on normal shutdown and are now included in orphan sweeping.

Dependency note:

- `tungstenite` v0.29.0 is now a direct dependency for the blocking local CDP WebSocket transport. License: MIT OR Apache-2.0.

Remaining I19e gaps are still substantial: Chrome/profile import, login start/finish/cancel lifecycle, current-tab attach, richer process diagnostics, screenshot/debug artifacts, and broader rendered-SPA parity. The new CDP path is intentionally scoped to fallback extraction with explicit session state and a throwaway profile.

Validation:

- `cargo test browser_cdp`
- `cargo test session::store::tests::orphan_sweep`
- `cargo test --test mock_site_cli owned_browser_fallback_replays_cookie_backed_session_without_agent_browser`
- `cargo test --test mock_site_cli owned_browser_fallback_renders_local_storage_backed_session_with_chrome -- --ignored` passed locally with system Chrome and confirmed localStorage-driven rendered DOM extraction.
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The CDP command construction and cookie-only fallback path are covered by deterministic tests, and the ignored local Chrome smoke test passed on this machine. Full migration confidence still requires more lifecycle/error-path coverage before switching defaults.

### D61: I19d uses owned CDP rendering for localStorage-backed primary extraction

The D60 renderer exposed a follow-up correctness gap: once `OwnedExtractorBackend` becomes default, a localStorage-backed request could otherwise return a static app shell successfully and never invoke browser fallback. I19d now routes owned primary extraction through the same temporary Chrome/CDP renderer whenever composed session state contains localStorage origins. Cookie-only extraction stays on the static HTTP path.

This still does not make every JavaScript-heavy cookie-backed page render through Chrome; there is no reliable generic signal for that yet. The new rule only covers the explicit structured-state case where static HTTP cannot replay localStorage at all.

Validation:

- `cargo test --test mock_site_cli owned_extractor_backend_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test --test mock_site_cli owned_browser_fallback_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The local Chrome smoke test proves rendered DOM extraction for the primary owned backend on this machine; broader rendered-JavaScript default policy remains an open I19d/I19f decision.

### D62: I19d retries owned extraction through CDP when CSS waits need rendered DOM

I19d now covers a second narrow rendered-JavaScript case without adding public API: when the owned static HTTP path cannot find a requested CSS `--wait-for-selector`, it retries the same extraction through the owned Chrome/CDP renderer. This mirrors the current Crawl4AI contract for CSS waits while preserving the existing safety boundary: JavaScript wait expressions are still rejected, and the only user input evaluated in Chrome is a JSON-quoted CSS selector passed to `document.querySelector(...)`.

This is deliberately not a blanket browser-rendering default. Static pages with matching selectors still stay on the faster HTTP path, and JavaScript-heavy pages without a wait selector remain a future policy/default decision for I19f.

Validation:

- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_local_storage_backed_session_with_chrome -- --ignored`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The ignored Chrome smoke test proves the delayed-DOM wait path locally, and the normal suite keeps static wait-selector behavior covered without requiring Chrome.

### D63: I19d adds first owned markdown table rendering

Before this slice, the owned markdown renderer collapsed HTML tables into plain text. Crawl4AI's markdown behavior and tests treat tables as part of the markdown-quality target, so the owned renderer now emits GitHub-flavored markdown tables for static table structures. Header rows are detected from `<th>` cells; tables without explicit headers use the first row as the markdown header. Cell content goes through the same inline renderer as normal text, so links are still resolved against the page base URL, and pipe characters inside cells are escaped.

This is not a full markdown/readability replacement yet. Remaining quality gaps still include captions, complex row/column spans, Crawl4AI-style citations/references, fit markdown, and broader main-content cleanup.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The static parity test now covers a table with links and literal pipe characters, but complex table semantics remain an explicit follow-up.

