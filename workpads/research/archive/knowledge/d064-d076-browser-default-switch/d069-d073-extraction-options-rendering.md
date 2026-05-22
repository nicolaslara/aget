# D69-D73: Extraction Options And Rendering

### D69: I19d adds conservative default main-content selection

Before this slice, owned extraction without an explicit selector formatted the parsed document root for text, markdown, and JSON content. That kept the implementation simple, but it meant default agent-facing output could include header, global nav, sidebar, or footer chrome even when the page had a single obvious content container.

I19d inspected Crawl4AI's cleaned-content flow again before changing this behavior:

- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py` removes excluded tags/selectors, builds a `content_element` from `css_selector` or `target_elements` when supplied, and serializes that cleaned element.
- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py` feeds `cleaned_html` into markdown generation by default.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` treats markdown generation as a separate layer over the selected cleaned HTML.

The owned backend now applies a conservative default content heuristic only when there is no explicit selector, no browser-fallback selector, no CSS wait selector, and the requested output format is text, markdown, or JSON. It prefers a unique `main`, then a unique `[role="main"]`, then a unique `article`, then falls back to `body` or the root document. HTML output without a selector still returns the cleaned document shape for debugging/compatibility, and explicit selectors keep their existing exact behavior.

This is not full readability pruning. It does not score competing article candidates, remove in-content nav, generate Crawl4AI citations, or apply fit-markdown filtering. It is a narrow default cleanup that reduces obvious page chrome while keeping wait-driven rendered extraction from accidentally discarding the waited element.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high for this slice. The behavior is deterministic and covered by a local fixture; broader readability quality remains an explicit I19d gap.

### D70: I19d supports safe `crawl4ai.excluded_tags` in the owned extractor

The I19c parity matrix keeps the current namespaced Crawl4AI backend-option surface alive until an `aget`-owned option namespace is designed. Before this slice, `OwnedExtractorBackend` rejected every backend option, which would make a default switch fail even for safe cleanup options that do not execute JavaScript or require browser-specific timing.

Before porting the option, I19d inspected `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: Crawl4AI reads `excluded_tags` from the run config, removes matching tag elements before selector-based content selection and cleaned-HTML serialization, then passes that cleaned HTML into the later markdown layer.

The owned backend now accepts only `crawl4ai.excluded_tags` from the backend-option escape hatch. The value is parsed like the helper's comma-separated list, but each entry must be a plain HTML tag name so the option cannot become a general CSS selector injection path. The removal happens before explicit `--exclude-selector` and before owned text/markdown/json/html formatting. Unsupported backend options still fail explicitly and point callers at the single supported option.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. This closes one safe backend-option parity gap with deterministic coverage. At this slice, other Crawl4AI options such as `only_text`, `word_count_threshold`, `wait_until`, `page_timeout`, `wait_for_timeout`, and `wait_for_images` remained unsupported until they had clear owned semantics.

### D71: I19d supports safe `crawl4ai.target_elements` in the owned extractor

The next safe backend-option slice ports `crawl4ai.target_elements`, again without copying upstream code. Crawl4AI's `content_scraping_strategy.py` applies `target_elements` after optional `css_selector` narrowing by collecting matches from the current content source and serializing only those elements into cleaned HTML. This option is selector-based content narrowing, not JavaScript execution.

The owned backend now accepts `crawl4ai.target_elements` as a comma-separated CSS selector list. Selectors are parsed before fetching or rendering so invalid CSS fails early. If a normal `--selector` is also present, target selectors are evaluated inside that selected source; otherwise they are evaluated against the parsed document root. Markdown rendering now includes the selected element itself rather than only its children, so targeting a heading preserves heading syntax instead of flattening it to plain text.

Unsupported backend options still fail explicitly. At this slice, the owned backend supports `crawl4ai.excluded_tags` and `crawl4ai.target_elements` from the Crawl4AI namespace.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. This closes another deterministic option-parity gap. At this slice, it intentionally did not port `only_text`, `word_count_threshold`, `wait_until`, `page_timeout`, `wait_for_timeout`, or `wait_for_images` yet.

### D72: I19d renders script-bearing pages through owned CDP by default

Before this slice, the owned extractor rendered through Chrome only for localStorage-backed session state or when a CSS wait selector was missing from the static HTML. That still left a major Crawl4AI parity gap: ordinary JavaScript-rendered pages without an explicit wait selector could return a static app shell successfully and never reach the owned CDP renderer.

Before changing the policy, I19d inspected Crawl4AI's render timing in `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py` and `references/repos/crawl4ai/crawl4ai/async_configs.py`. Crawl4AI navigates in a browser, applies any configured `wait_for`, then waits `delay_before_return_html` before reading final HTML; the default delay is 0.1 seconds.

The owned extractor now validates owned options first, performs the fast Rust HTTP fetch, and escalates to the owned CDP renderer when the static response contains executable script tags. The CDP renderer now also waits 100 ms after navigation and any CSS wait before reading `document.documentElement.outerHTML`, matching Crawl4AI's default pre-return delay at a narrow level. This keeps static pages on the fast path while covering a concrete class of client-rendered pages without requiring callers to guess a wait selector.

This is still not full smart load detection. It does not wait for network idle, long async chains, virtual scrolling, image readiness, or app-specific readiness signals. Pages with scripts now require local Chrome/Chromium when using the owned backend, which is acceptable before I19f but needs to be reflected in default-runtime docs before the switch.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_scripted_page_without_wait_with_chrome -- --ignored`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`

Confidence: Medium. The local Chrome smoke proves the new no-wait script rendering path, but the readiness heuristic remains intentionally simple and should be expanded or documented before making the owned backend the default.

### D73: I19d ports `crawl4ai.delay_before_return_html` to owned CDP rendering

D72 hard-coded Crawl4AI's default 0.1 second pre-return delay in the owned CDP renderer. The next small parity step makes the public `crawl4ai.delay_before_return_html` backend option work on the owned backend as well. The source behavior remains `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI sleeps after `wait_for` and before retrieving final HTML, and `references/repos/crawl4ai/crawl4ai/async_configs.py`, where the default is 0.1 seconds.

The owned extractor now parses `crawl4ai.delay_before_return_html` as a non-negative finite number of seconds, defaults to 0.1 seconds, and passes the resulting duration into `browser_cdp::render_page`. The delay is applied after navigation and any CSS wait selector, immediately before reading `document.documentElement.outerHTML`. Unsupported backend options still fail explicitly; the supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, and `delay_before_return_html`.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_honors_render_delay_option_with_chrome -- --ignored`

Confidence: Medium-high. The option is narrow, typed, and covered by an ignored Chrome smoke with a delayed client render; broader smart readiness remains open.
