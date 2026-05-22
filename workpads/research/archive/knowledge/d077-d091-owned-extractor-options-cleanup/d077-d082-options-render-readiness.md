# D77-D82: Options And Render Readiness

### D77: I19d ports `crawl4ai.only_text` to owned markdown rendering

The next safe Crawl4AI option to port was `crawl4ai.only_text`. Before changing the owned extractor, I19d inspected the upstream behavior in `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where `only_text` replaces text-formatting inline tags from `ONLY_TEXT_ELIGIBLE_TAGS` with their text content, and `references/repos/crawl4ai/crawl4ai/config.py`, where that tag allowlist is defined. The local command helper already parsed this option as a boolean.

The owned extractor now parses `crawl4ai.only_text` with the same boolean spelling set used by the command helper (`true/false`, `1/0`, `yes/no`, `on/off`). When enabled, owned markdown rendering treats Crawl4AI's text-formatting inline tags such as `strong`, `em`, `code`, `span`, `mark`, and `time` as plain text while preserving structural markdown such as headings, lists, links, tables, and preformatted code blocks. This mirrors the safe part of Crawl4AI's option without adding JavaScript execution or broader cleanup policy.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, and `delay_before_return_html`. `word_count_threshold`, `wait_until`, `page_timeout`, `wait_for_timeout`, and `wait_for_images` remained unsupported at this slice until they had owned semantics and validation strong enough for authenticated-session use.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. This is a narrow renderer option with deterministic fixture coverage; it does not change broader readability or rendered-page readiness behavior.

### D78: I19d ports `crawl4ai.page_timeout` and `crawl4ai.wait_for_timeout` to owned CDP rendering

The next render-control slice ports the timeout options that have clear browser-operation semantics in Crawl4AI. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `page_timeout` is an integer millisecond timeout for page operations and `wait_for_timeout` optionally overrides the timeout used for `wait_for`, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where `wait_for_timeout` falls back to `page_timeout` when absent.

The owned extractor now parses `crawl4ai.page_timeout` and `crawl4ai.wait_for_timeout` as non-negative integer millisecond values. `page_timeout` is applied to CDP page creation, domain enabling, state loading, navigation, and final DOM reads in `browser_cdp::render_page`; browser process startup still uses the outer `aget --timeout` budget. `wait_for_timeout` applies only to the CSS selector wait and falls back to the page timeout when absent. The CSS-only wait safety rule remains unchanged.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, `delay_before_return_html`, `page_timeout`, and `wait_for_timeout`. `word_count_threshold`, `wait_until`, and `wait_for_images` remained unsupported at this slice until they had owned semantics and validation strong enough for authenticated-session use.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`

Confidence: Medium-high. The parser is deterministically covered, and the ignored Chrome smoke covers the positive rendered wait path with explicit page and wait timeouts. This does not add network-idle or image-readiness behavior.

### D79: I19d ports bounded `crawl4ai.wait_until` support to owned CDP rendering

The next render-control slice ports the part of `crawl4ai.wait_until` that has direct owned CDP semantics. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where the default `wait_until` is `domcontentloaded`, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI passes that value into Playwright navigation.

The owned renderer now accepts `crawl4ai.wait_until=domcontentloaded` and `crawl4ai.wait_until=load`. These map to CDP `Page.domContentEventFired` and `Page.loadEventFired` respectively. Unsupported values such as `networkidle` fail explicitly instead of being ignored, because the owned renderer does not yet implement network-idle tracking. The default owned behavior remains the existing load-event wait until a broader readiness policy is chosen.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, `delay_before_return_html`, `page_timeout`, `wait_for_timeout`, and bounded `wait_until`. `word_count_threshold` and `wait_for_images` remained unsupported at this slice until they had owned semantics and validation strong enough for authenticated-session use.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_renders_waited_javascript_page_with_chrome -- --ignored`

Confidence: Medium-high. This covers the two CDP lifecycle events the owned renderer can currently prove. It intentionally does not claim Playwright `networkidle` parity.

### D80: I19d ports `crawl4ai.wait_for_images` to owned CDP rendering

The next render-readiness slice ports the safe part of `crawl4ai.wait_for_images`. Before changing the owned renderer, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `wait_for_images` is a boolean navigation/timing option defaulting to false, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI waits for `domcontentloaded`, sleeps briefly, then checks that all `<img>` elements are complete with a one-second timeout. Crawl4AI logs a warning and continues if images do not finish.

The owned extractor now parses `crawl4ai.wait_for_images` as a boolean using the same boolean spelling set as other owned options. When enabled, it forces the owned CDP rendering path, waits up to one second for `Array.from(document.images).every((img) => img.complete)`, and continues with an agent-visible warning if the image wait times out. This remains a browser-readiness option only; it does not add image description extraction, screenshot capture, external image fetching outside the browser, or JavaScript waits from user input.

The supported owned Crawl4AI namespace is now `excluded_tags`, `target_elements`, `only_text`, `delay_before_return_html`, `page_timeout`, `wait_for_timeout`, bounded `wait_until`, and `wait_for_images`. `word_count_threshold` remained unsupported at this slice until source inspection established whether the current Crawl4AI default scraper applies it to the behavior `aget` actually depends on.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_honors_wait_for_images_option_with_chrome -- --ignored`

Confidence: Medium. The behavior is source-faithful for the explicit image-completion wait and covered by a local Chrome smoke, but broader rendered-page readiness remains a larger I19d gap.

### D81: I19d accepts `crawl4ai.word_count_threshold` with current Crawl4AI default semantics

The last Crawl4AI helper option still rejected by the owned extractor was `crawl4ai.word_count_threshold`. Before changing the owned backend, I19d inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `CrawlerRunConfig` accepts and stores `word_count_threshold`, and `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`, where the default `LXMLWebScrapingStrategy._scrap` receives that parameter but the cleaned-content path currently calls `remove_empty_elements_fast(body, 1)` with a hardcoded threshold. The upstream regression tests also cover config serialization/defaults and browser-context reuse for varying `word_count_threshold`, not output pruning in the default markdown path.

The owned extractor now accepts and validates `crawl4ai.word_count_threshold` as an integer so existing command-helper callers can switch to the owned backend without hitting an unsupported-option failure. It intentionally does not use the value to prune output because that would be stricter than the inspected Crawl4AI default path. Unsupported backend options still fail explicitly.

At this point the owned backend accepts every namespaced Crawl4AI option that the PoC command helper allowed: `excluded_tags`, `target_elements`, `only_text`, `word_count_threshold`, `wait_until`, `page_timeout`, `wait_for_timeout`, `delay_before_return_html`, and `wait_for_images`.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high. This closes the compatibility-option gap without inventing behavior upstream does not currently prove; the remaining I19d gaps are broader quality/readiness work rather than a helper-option mismatch.

### D82: I19d ports bounded `crawl4ai.wait_until=networkidle` support to owned CDP rendering

The next rendered-readiness slice completes the safe `wait_until` value set that the PoC helper exposed. Before changing the owned renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/async_configs.py`, where `CrawlerRunConfig.wait_until` is the navigation wait condition, and `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`, where Crawl4AI passes that value into Playwright navigation before later image waits or HTML capture.

The owned CDP renderer now accepts `crawl4ai.wait_until=networkidle`. It sends `Page.navigate` without discarding interleaved CDP events, requires the navigation response plus `Page.domContentEventFired`, tracks same-target `Network.requestWillBeSent`, `Network.loadingFinished`, and `Network.loadingFailed` events, and considers the page idle after there are no in-flight tracked requests for 500 ms. This is a bounded CDP implementation of the Playwright concept, not a broader smart-readiness system: long-polling, websockets, service-worker behavior, virtual scrolling, and app-specific readiness still belong to later I19d work.

The supported owned `crawl4ai.wait_until` values are now `domcontentloaded`, `load`, and `networkidle`.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli owned_extractor_backend_honors_networkidle_wait_until_with_chrome -- --ignored`

Confidence: Medium. The local Chrome smoke proves the owned network-idle wait for a delayed same-origin fetch, but this should still be treated as bounded parity rather than full Playwright readiness equivalence.
