# D355: Unlabeled Density Main Content

## Decision

Owned main-content extraction now lets dense unlabeled `div` and `section` candidates win when Crawl4AI-style pruning signals are strong enough.

## Source Evidence

- `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`, `crawl4ai/content_filter_strategy.py`.
- `PruningContentFilter` scores structural tags using text density, non-link density, tag weight, class/id weight, and text length.
- The source scoring does not require positive `class`, `id`, `role`, or `aria-label` content labels before a generic structural tag can remain after pruning.

## Implementation

- `src/extraction/owned/content/main_content.rs` now keeps the existing positive-label path but also accepts unlabeled `div`/`section` candidates whose Crawl4AI-like pruning score is high.
- The generic unlabeled path still rejects candidates with negative class/id labels and candidates inside page-chrome ancestors.
- Broad unlabeled layout wrappers are not promoted when they contain a more specific descendant content container with positive labels.
- Added deterministic mock-site coverage for an unlabeled dense content container beating nav/footer body fallback.
- No site-specific labels or built-in site behavior were added.

## Validation

- `cargo test aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

## Follow-Up

- I19d still remains open for fuller Crawl4AI-quality readability/markdown and richer rendered-page readiness.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.
