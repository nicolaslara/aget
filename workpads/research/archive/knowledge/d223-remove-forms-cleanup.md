# D223: Remove Forms Cleanup

Date: 2026-05-21

Source inspected:

- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `CrawlerRunConfig.remove_forms` is documented as removing all `<form>` elements and defaults to `false`.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: both content-scraping paths remove descendant `<form>` nodes when `remove_forms` is enabled, before normal content processing continues.
- `scripts/crawl4ai_extract.py`: the compatibility helper keeps an explicit allowlist for accepted `crawl4ai.*` options before importing or invoking Crawl4AI.

Decision:

- Add owned support for `crawl4ai.remove_forms` as a boolean backend option with default `false`.
- Remove `<form>` elements during owned pre-content cleanup only when the option is true.
- Keep form content visible by default so existing extraction behavior does not change without an explicit caller option.
- Align the Crawl4AI command compatibility allowlist so existing callers can pass the same option through the command adapter.

Boundary:

- This is generic document cleanup, not site-specific login/paywall handling.
- The option does not execute caller-provided JavaScript and does not change the CSS-only wait safety boundary.

Validation:

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options -- --nocapture`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
