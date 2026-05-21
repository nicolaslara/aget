# D236: Main-Content Word Threshold

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`: `PruningContentFilter._compute_composite_score` returns `-1.0` when `min_word_threshold` is set and a node's word count is below that threshold, guaranteeing removal before normal scoring.
- `src/extraction/owned/options.rs`: owned extraction already parses `crawl4ai.word_count_threshold`, with the same default of `1` used by the local compatibility surface.

## Decision

- Keep the default `crawl4ai.word_count_threshold=1` behavior unchanged.
- Apply explicit `crawl4ai.word_count_threshold` as a hard lower bound for default main-content candidates before scoring.
- Keep explicit `--selector` and `crawl4ai.target_elements` behavior unchanged; this threshold only affects automatic default main-content candidate selection.
- Preserve the existing cleaned-HTML leaf-pruning behavior that already uses the same threshold after a root is selected.

## Validation

- `cargo fmt --check`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
