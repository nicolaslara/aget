# D266: Crawl4AI Internal Link Cleanup Option

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `CrawlerRunConfig.exclude_internal_links` is exposed as a boolean option with default `False` and serialized with the other crawler options.
- `references/repos/crawl4ai/crawl4ai/utils.py`: Crawl4AI classifies special-scheme URLs such as `mailto:`, `tel:`, `ftp:`, `file:`, `data:`, and `javascript:` as external, treats relative URLs as internal, and compares normalized hosts against the page base domain for internal/external link classification.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: link processing uses the same base-domain classification and removes external links when `exclude_external_links` is enabled. This owned slice mirrors that classification for the exposed internal-link cleanup option.

## Decision

Add owned support for `crawl4ai.exclude_internal_links` as a boolean backend option with default `false`.

When enabled, owned HTML cleanup removes `<a href>` elements whose target is relative or resolves to the same Crawl4AI-style base domain. External links, including special-scheme links that Crawl4AI treats as external, are preserved by this option.

## Implementation

- `src/extraction/owned/options.rs` parses and advertises `crawl4ai.exclude_internal_links`.
- `src/extraction/html_clean/urls.rs` adds internal-link removal using the existing Crawl4AI-like base-domain helper and external URL classifier.
- `src/extraction/owned/page.rs` applies internal-link cleanup before owned content extraction.
- `scripts/crawl4ai_extract.py` and `tests/support/bin/aget_mock_backend/config.rs` allow the option for command-compatibility forwarding and test validation.
- `README.md` documents the option in the active `AgetExtractor` compatibility list.

## Validation

- `cargo test extraction::html_clean::tests::internal_link_cleanup_removes_relative_and_same_base_domain_links`
- `cargo test --test mock_site_cli aget_extractor::aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli validation::get_command_backend_accepts_scan_full_page_options`
- `cargo fmt --check && cargo test && git diff --check`
