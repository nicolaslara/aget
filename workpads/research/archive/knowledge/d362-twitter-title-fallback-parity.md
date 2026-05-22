# D362: Twitter Title Fallback Parity

## Decision

Owned extractor metadata coverage now locks the Crawl4AI-compatible title fallback order when a page has no `<title>` and no `og:title`: `twitter:title` becomes the page `title` metadata.

## Source Evidence

- `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`.
- `crawl4ai/content_scraping_strategy.py` calls `extract_metadata_using_lxml` before content filtering.
- `crawl4ai/utils.py` `extract_metadata_using_lxml` first tries `<title>`, then `og:title`, then `twitter:title`.

## Implementation

- Added a deterministic mock-site fixture with no `<title>` and no `og:title`, but with `twitter:title`.
- Added owned extractor coverage proving page content remains unchanged, `page_metadata.title` falls back to `twitter:title`, and no Open Graph title key is invented.
- No runtime behavior changed.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.

## Validation

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

## Follow-Up

- I19d remains open for fuller Crawl4AI-quality markdown/readability and richer rendered-page readiness.
