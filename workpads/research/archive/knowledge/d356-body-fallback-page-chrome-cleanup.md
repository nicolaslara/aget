# D356: Body Fallback Page Chrome Cleanup

## Decision

Owned main-content extraction removes generic page-chrome tags when automatic main-content selection falls back to `body` or the document root.

## Source Evidence

- `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`, `crawl4ai/content_filter_strategy.py`.
- `PruningContentFilter` removes comments and unwanted tags before pruning the tree.
- Its unwanted tags include `nav`, `footer`, `header`, `aside`, `script`, `style`, `form`, `iframe`, and `noscript`.

## Implementation

- `src/extraction/owned/content.rs` now detects the automatic body/root fallback case for main-content extraction.
- In that fallback only, it removes `nav`, `footer`, `header`, `aside`, `form`, `iframe`, and `noscript` before markdown/text serialization.
- Explicit selectors, explicit HTML output, and already-selected `main`/`article`/container candidates keep their existing behavior.
- Existing earlier cleanup already removes `script` and `style`, so this slice reuses that boundary.
- Added deterministic mock-site coverage for a bare body page where title and paragraph remain while header/nav/footer are removed.

## Validation

- `cargo test aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

## Follow-Up

- I19d still remains open for fuller Crawl4AI-quality readability/markdown and richer rendered-page readiness.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.
