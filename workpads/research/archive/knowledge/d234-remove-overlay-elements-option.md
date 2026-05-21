# D234: Explicit Overlay Cleanup Option

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `CrawlerRunConfig.remove_overlay_elements` is a boolean option, defaulting to `false` in upstream Crawl4AI.
- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`: when `remove_overlay_elements` is true, Crawl4AI runs overlay-removal JavaScript before HTML capture and continues on failure.
- `scripts/crawl4ai_extract.py`: the aget PoC compatibility helper set `remove_overlay_elements=True` by default, making overlay cleanup part of the behavior aget already used from Crawl4AI.

## Decision

- Preserve aget's current default overlay cleanup behavior in the owned extractor.
- Add `crawl4ai.remove_overlay_elements` as an explicit boolean owned backend option so callers can opt out with `false`.
- Add the same option to the Crawl4AI command compatibility allowlist and mock backend validation.
- Keep the option generic; do not add site-specific consent, paywall, or login popup handling.

## Validation

- `cargo fmt --check`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options -- --nocapture`
