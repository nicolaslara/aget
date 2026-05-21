# D217: CSS Wait Prefix Normalization

Date: 2026-05-21

Source inspected:

- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`: `smart_wait` treats `css:<selector>` as an explicit CSS selector and strips the prefix before calling Playwright `page.wait_for_selector`.

Decision:

- Preserve `aget`'s v1 safety boundary that JavaScript waits are rejected for authenticated-session safety.
- Normalize explicit `css:` waits by stripping the prefix before owned CDP `document.querySelector` expressions.
- Keep plain CSS selectors unchanged.
- Preserve current output metadata: the recorded request option remains what the caller supplied.

Boundary:

- Static owned extraction already uses `parse_css_selector`, which strips `css:` before selector parsing. This change closes the rendered/CDP wait path.
- This does not add Crawl4AI's JavaScript wait fallback; that remains intentionally unsupported in v1.
- I19d remains open for broader Crawl4AI-quality markdown/readability and richer rendered-page readiness.

Validation:

- `cargo fmt --check`
- `cargo test browser_cdp::tests::scripts::selector_wait_expression_strips_explicit_css_prefix`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test`
- `git diff --check`
