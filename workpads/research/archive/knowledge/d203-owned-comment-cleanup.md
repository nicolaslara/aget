# D203: Owned HTML comment cleanup

## Decision

`AgetExtractor` now removes HTML comment nodes during owned cleanup before selector/output shaping.

Primary source:

- `references/repos/crawl4ai/crawl4ai/utils.py`
- `references/repos/crawl4ai/crawl4ai/content_filter_strategy.py`

Crawl4AI removes BeautifulSoup comment nodes during content cleanup and pruning. The owned extractor already ignored comments during markdown rendering, but `--content-format html` could retain comment-only debug/private text because HTML output serializes the cleaned DOM. This slice removes comment nodes from the owned DOM cleanup path.

## Boundaries

- Comment nodes are removed before script/style/link/meta/noscript removal, wait-selector checks, overlay cleanup, excluded tags, and output rendering.
- Markdown and text behavior remains unchanged because comments were not rendered as text.
- HTML output now matches the cleanup expectation more closely and avoids retaining comment-only private/debug strings.
- This is local deterministic cleanup; no site-specific rules or hosted services are introduced.

## Validation

- Added deterministic mock-site coverage proving cleaned HTML output omits a comment-only `debug-secret` string.
- `cargo fmt --check`
- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test extraction::`
- `cargo test`
- `git diff --check`
