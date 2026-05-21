# D226: Keep Data Attributes Cleanup

Date: 2026-05-21

Source inspected:

- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `CrawlerRunConfig.keep_data_attributes` is documented as retaining `data-*` attributes during cleanup and defaults to `false`.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: `remove_unwanted_attributes_fast` keeps normal important attributes, and when `keep_data_attributes=True` also retains attributes whose names start with `data-`.
- `scripts/crawl4ai_extract.py`: the compatibility helper keeps an explicit allowlist for accepted `crawl4ai.*` options before invoking Crawl4AI.

Decision:

- Add owned support for `crawl4ai.keep_data_attributes` as a boolean backend option with default `false`.
- Preserve `data-*` attributes in owned cleaned HTML only when the option is true.
- Keep selectors running before attribute pruning, so callers can still select by `data-*` attributes even when the cleaned output omits those attributes.
- Align the Crawl4AI command compatibility helper and checked-in mock backend validation allowlists.

Boundary:

- This only affects serialized cleaned HTML attributes. Markdown and text extraction behavior is unchanged.
- Non-data unimportant attributes such as `style`, event handlers, ARIA labels, and `rel` remain pruned.
- This is an explicit caller option because `data-*` values may include application-specific identifiers.

Validation:

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options -- --nocapture`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
