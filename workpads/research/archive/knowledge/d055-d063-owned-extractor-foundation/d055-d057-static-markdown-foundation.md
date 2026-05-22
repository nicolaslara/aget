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

