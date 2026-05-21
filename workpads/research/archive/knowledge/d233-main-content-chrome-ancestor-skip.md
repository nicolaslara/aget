# D233: Main Content Page-Chrome Ancestor Skip

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`: `RelevantContentFilter.excluded_tags` includes generic page chrome and non-content tags such as `nav`, `footer`, `header`, `aside`, `script`, `style`, `form`, `iframe`, and `noscript`.
- `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`: `PruningContentFilter.filter_content` removes comments and unwanted tags before pruning and returning content blocks.

## Decision

- Keep explicit `--selector` and `crawl4ai.target_elements` behavior unchanged.
- For default owned main-content selection only, skip candidate `main`, role-main, `article`, `section`, and `div` elements whose ancestors are Crawl4AI pruning-filter excluded page-chrome tags.
- Preserve the existing fallback to `body` or root when no positive main-content candidate remains.

## Validation

- `cargo fmt --check`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
