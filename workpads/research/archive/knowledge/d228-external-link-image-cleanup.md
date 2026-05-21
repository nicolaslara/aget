# D228: Crawl4AI External Link And Image Cleanup Options

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `CrawlerRunConfig.exclude_external_links` and `CrawlerRunConfig.exclude_external_images` default to `false` and are serialized as crawler options.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: scraping removes external `<a href>` nodes when `exclude_external_links` is true and removes external `<img src>` nodes when `exclude_external_images` is true.
- `references/repos/crawl4ai/crawl4ai/utils.py`: external URL detection treats special schemes such as `mailto:`, `data:`, and `javascript:` as external, treats relative URLs as internal, strips `www.`, and compares against a base-domain string.

## Decision

- Add owned support for `crawl4ai.exclude_external_links` and `crawl4ai.exclude_external_images` as boolean backend options with default `false`.
- Remove matching external link/image elements before owned HTML/markdown/text/json content extraction, after the owned base URL is resolved from the final URL and any `<base href>`.
- Preserve same-origin relative URLs and absolute URLs under the same Crawl4AI-like base domain.
- Keep the generic fetcher boundary: this is URL/domain cleanup only, not site-specific link policy.

## Validation

- `cargo fmt --check`
- `cargo test external_ -- --nocapture`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options -- --nocapture`
