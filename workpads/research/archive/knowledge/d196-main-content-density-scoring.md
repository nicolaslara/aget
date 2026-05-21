# D196: Main Content Density Scoring

The owned default main-content candidate scorer now includes deterministic signals inspired by Crawl4AI's `PruningContentFilter` in `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py` at commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`.

Source-backed signals used:

- Text density: text length relative to serialized element size.
- Link density: candidates with mostly link text receive less score.
- Tag weights: semantic containers such as `article` and `main` remain preferred.
- Text length: longer useful content receives a modest logarithmic boost.

The implementation remains a local deterministic heuristic in `src/extraction/owned/content.rs`; it does not copy Crawl4AI code, introduce LLM filtering, or add site-specific rules. Existing label bonuses/penalties remain in place so explicit `content`/`article` style labels still help candidate selection.

New coverage:

- `tests/mock_site_cli/aget_extractor_site.rs` adds `/main-content-link-density`, where a link-heavy `article` competes with a denser `div#content`.
- `tests/mock_site_cli/aget_extractor.rs` verifies the default markdown extraction selects the denser article body.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test aget_extractor::tests`
- `cargo test`
- `git diff --check`
