# D239: Empty-Pruning Threshold Boundary

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: after `only_text` replacement and base64 image cleanup, Crawl4AI calls `remove_empty_elements_fast(body, 1)` before unwanted-attribute pruning.
- `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`: `PruningContentFilter` applies `min_word_threshold` to content-filter scoring, which is a separate readability/candidate-filtering boundary from cleaned-HTML empty-leaf pruning.

## Decision

- Keep owned `crawl4ai.word_count_threshold` as a default main-content candidate constraint.
- Stop passing `crawl4ai.word_count_threshold` into owned empty-leaf cleanup; cleaned-HTML empty pruning now uses Crawl4AI's fixed threshold of `1`.
- Preserve the existing code-block whitespace bypass, so whitespace-only spans inside `pre`/`code` are still retained.

## Validation

- `cargo fmt --check`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `git diff --check`
- `cargo test`
