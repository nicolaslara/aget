# D231: Crawl4AI Custom Social Media Domains

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `CrawlerRunConfig.exclude_social_media_domains` is a list-valued crawler option and defaults to Crawl4AI's built-in social-domain list when unset.
- `references/repos/crawl4ai/crawl4ai/config.py`: `SOCIAL_MEDIA_DOMAINS` defines the built-in social-domain list.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: custom social domains are only applied through the `exclude_social_media_links` gate; when enabled, Crawl4AI merges configured social domains with the built-in list before adding them to link-domain exclusions.

## Decision

- Add owned support for `crawl4ai.exclude_social_media_domains` as a list backend option.
- Apply custom social domains only when `crawl4ai.exclude_social_media_links=true`.
- Keep built-in social domains active even when custom domains are supplied.
- Keep images untouched by this link-specific option; image/domain options remain responsible for image removal.
- Align the Crawl4AI command compatibility allowlist and mock backend validation with the new option.

## Validation

- `cargo fmt --check`
- `cargo test social_media -- --nocapture`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options -- --nocapture`
