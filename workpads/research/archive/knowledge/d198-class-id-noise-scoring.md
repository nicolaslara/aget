# D198: Class/id noise scoring for main-content candidates

## Decision

`AgetExtractor` main-content selection now applies a generic class/id noise penalty inspired by Crawl4AI `PruningContentFilter`.

Primary source:

- `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`
- `RelevantContentFilter.negative_patterns`
- `PruningContentFilter._compute_class_id_weight`

The local scorer already used transparent positive labels and link/text-density signals. This slice adds an explicit penalty when candidate-level `class` or `id` starts with Crawl4AI-style noisy labels such as `nav`, `footer`, `header`, `sidebar`, `ads`, `comment`, `promo`, `advert`, `social`, or `share`.

## Boundaries

- This is a deterministic heuristic, not a full port of Crawl4AI pruning.
- The rule is generic and site-neutral.
- The penalty applies only to candidate element `class`/`id`; it does not inspect descendant labels or introduce JavaScript execution.

## Validation

- `cargo fmt --check`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test`
- `git diff --check`
