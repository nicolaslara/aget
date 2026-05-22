# D372: Crawl4AI `process_in_browser` Local-Content Routing

Date: 2026-05-23

## Decision

Owned extraction accepts `crawl4ai.process_in_browser` and uses it to route `raw:`, `raw://`, and `file://` inputs through the owned CDP browser pipeline when requested.

The existing fast static local-content path remains the default. Safe browser-only local options such as CSS waits, image readiness, iframe processing, and full-page scanning also route local inputs through the browser path. Arbitrary user-supplied JavaScript remains unsupported.

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/async_configs.py` documents `process_in_browser`, defaults it to `False`, stores it on `CrawlerRunConfig`, and serializes it through `dump()`.
- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py` keeps raw/file inputs on a fast direct HTML path by default, but routes them through `_crawl_web()` when `process_in_browser` or browser-only options such as waits, screenshots, scanning, iframe processing, or JavaScript are present.
- The same Crawl4AI source loads browser-routed raw/file content with `page.set_content()` instead of normal network navigation.

## Implementation Notes

- `src/extraction/owned/options.rs` stores `process_in_browser`.
- `src/extraction/owned/options/apply.rs` validates `crawl4ai.process_in_browser` as a boolean.
- `src/extraction/owned/page.rs` materializes raw HTML into a temporary private run file URL before CDP rendering and preserves the original raw/file final URL in the extraction result.
- `src/extraction/owned/page/rendered.rs` supports rendering one URL while overriding the final URL used by extraction output.
- The optional Crawl4AI compatibility helper forwards `process_in_browser` to real Crawl4AI when the installed version accepts it.

## Validation

Passed:

- `cargo test extraction::owned::page::tests::`
- `cargo test get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/crawl4ai_extract.py scripts/aget_crawl4ai_compat/options.py`
- Python helper parse smoke for `crawl4ai.process_in_browser=true`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

Added but not run in the deterministic gate:

- Ignored local Chrome smoke coverage for raw browser processing.
