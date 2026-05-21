# D230: Crawl4AI Social Media Link Cleanup Option

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `CrawlerRunConfig.exclude_social_media_links` defaults to `false`, `exclude_social_media_domains` defaults to `SOCIAL_MEDIA_DOMAINS`, and both are serialized as crawler options.
- `references/repos/crawl4ai/crawl4ai/config.py`: `SOCIAL_MEDIA_DOMAINS` includes Facebook, Twitter/X, LinkedIn, Instagram, Pinterest, TikTok, Snapchat, and Reddit domains.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: when `exclude_social_media_links` is true, Crawl4AI merges the social-domain list into `exclude_domains`; link processing then removes matching external anchors.

## Decision

- Add owned support for `crawl4ai.exclude_social_media_links` as a boolean backend option with default `false`.
- Remove matching social-media `<a href>` elements before owned content extraction when the option is true.
- Keep images untouched by this link-specific option; image/domain options remain responsible for image removal.
- Keep the default social link behavior unchanged.

## Validation

- `cargo fmt --check`
- `cargo test social_media -- --nocapture`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options -- --nocapture`
