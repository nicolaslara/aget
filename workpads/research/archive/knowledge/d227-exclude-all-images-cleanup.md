# D227: Exclude All Images Cleanup

Date: 2026-05-21

Source inspected:

- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `CrawlerRunConfig.exclude_all_images` defaults to `false` and is serialized as a crawler option.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: when `exclude_all_images` is true, content scraping removes all `<img>` elements before content processing.
- `scripts/crawl4ai_extract.py`: the compatibility helper keeps an explicit allowlist for accepted `crawl4ai.*` options before invoking Crawl4AI.

Decision:

- Add owned support for `crawl4ai.exclude_all_images` as a boolean backend option with default `false`.
- Remove `<img>` elements during owned pre-content cleanup only when the option is true.
- Keep default image behavior unchanged.
- Align the Crawl4AI command compatibility helper and checked-in mock backend validation allowlists.

Boundary:

- This option removes image elements from owned HTML, markdown, text, and JSON content extraction because it runs before content selection/rendering.
- It is generic document cleanup; it does not fetch, inspect, or classify image resources.

Validation:

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options -- --nocapture`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
