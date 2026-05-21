# D229: Crawl4AI Domain Exclusion Cleanup Option

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `CrawlerRunConfig.exclude_domains` defaults to an empty list and is serialized as a crawler option.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: scraping checks `exclude_domains` while processing links and images, removing matching-domain anchors and image elements.
- `references/repos/crawl4ai/crawl4ai/utils.py`: `get_base_domain` strips `www.`, handles common multi-part suffixes, and `is_external_url` treats relative URLs as internal.

## Decision

- Add owned support for `crawl4ai.exclude_domains` as a comma-separated list backend option with an empty default.
- Remove `<a href>` and `<img src>` elements whose Crawl4AI-like base domain matches one of the configured domains.
- Keep default link/image behavior unchanged and keep unrelated relative, same-domain, and external URLs unless their base domain is explicitly excluded.
- Normalize configured domains through the same lightweight Crawl4AI-like base-domain helper used for extracted URLs.

## Validation

- `cargo fmt --check`
- `cargo test excluded_domain -- --nocapture`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options -- --nocapture`
