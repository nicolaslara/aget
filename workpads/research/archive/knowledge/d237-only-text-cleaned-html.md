# D237: `only_text` Cleaned HTML

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/config.py`: `ONLY_TEXT_ELIGIBLE_TAGS` contains inline formatting and semantic text tags such as `strong`, `em`, `code`, `abbr`, `time`, `small`, and `mark`.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: when `only_text` is true, Crawl4AI replaces those eligible elements with text before base64 image cleanup, empty-element pruning, and attribute pruning.

## Decision

- Preserve existing owned markdown behavior for `crawl4ai.only_text=true`.
- Apply `crawl4ai.only_text=true` to owned cleaned HTML by replacing eligible inline descendants with text nodes before existing base64 cleanup, empty-element pruning, and attribute pruning.
- Preserve explicit selector and `crawl4ai.target_elements` roots when they directly select an eligible inline tag, and preserve eligible inline ancestors of those selected roots so cleanup does not invalidate selected-node IDs.

## Validation

- `cargo fmt --check`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `git diff --check`
- `cargo test`
