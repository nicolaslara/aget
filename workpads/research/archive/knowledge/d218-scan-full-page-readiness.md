# D218: Scan Full Page Readiness

Date: 2026-05-21

Source inspected:

- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `scan_full_page` defaults to `False`, `scroll_delay` defaults to `0.2`, and `max_scroll_steps` is exposed as optional config.
- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`: full-page scanning runs before pre-wait JavaScript/interactions, uses the page timeout as the scan timeout, scrolls by viewport height, applies a runtime max-scroll default of 10 when the config value is unset, returns to the bottom after scanning, and logs warnings while continuing after timeout/failure.
- `references/repos/crawl4ai/docs/md_v2/api/parameters.md`: documents `scan_full_page`, `scroll_delay`, and `max_scroll_steps` as page-interaction options for lazy/infinite-scroll content.

Decision:

- Add owned backend options `crawl4ai.scan_full_page`, `crawl4ai.scroll_delay`, and `crawl4ai.max_scroll_steps`.
- Preserve the source-backed defaults: scanning disabled unless requested, 200 ms scroll delay, and a bounded 10-step runtime default.
- Implement the owned CDP behavior as generic viewport-step scrolling before selector waits, image waits, overlay cleanup, and HTML capture.
- Continue extraction with a warning if the scan command fails or times out, matching Crawl4AI's non-fatal scan boundary.

Boundary:

- This does not add Crawl4AI `js_code`, interaction hooks, virtual scroll, screenshots, or iframe processing.
- The static HTTP path validates the options but does not attempt scrolling because no browser page exists.
- Current-tab and owned rendered URL extraction both receive the same scan options through the `AgetBrowser`/CDP seam.

Validation:

- `cargo check`
- `cargo test full_page_scan_expression_uses_bounded_viewport_scrolls -- --nocapture`
- `cargo test browser_cdp_render_attached_page_scans_full_page_before_capture -- --nocapture`
- `cargo test aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
