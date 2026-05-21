# D221: Ordered List Start Cleanup Boundary

Date: 2026-05-21

Source inspected:

- `references/repos/crawl4ai/crawl4ai/html2text/utils.py`: `list_numbering_start` reads `ol[start]` and falls back to `0` when the attribute is missing or malformed.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: list rendering initializes ordered-list numbering from `list_numbering_start(attrs)`.
- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py`: the default markdown input source is `cleaned_html`.
- `references/repos/crawl4ai/crawl4ai/config.py`: default `IMPORTANT_ATTRS` are `src`, `href`, `alt`, `title`, `width`, `height`, `class`, and `id`.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: cleaned HTML removes attributes not present in `IMPORTANT_ATTRS` unless explicitly configured to keep more.

Decision:

- Do not port `ol[start]` numbering as default owned extraction behavior yet.
- The html2text converter can use the attribute, but the active Crawl4AI default path used by `aget` passes cleaned HTML where `start` is stripped.
- Lock the owned cleanup boundary with deterministic coverage so ordered lists continue to start at `1` after default cleanup.

Boundary:

- This does not rule out a future explicit keep-attrs option.
- If `aget` later supports a Crawl4AI-compatible `keep_attrs=["start"]` or raw-HTML markdown source, ordered-list start preservation should be revisited with source-backed coverage.

Validation:

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
