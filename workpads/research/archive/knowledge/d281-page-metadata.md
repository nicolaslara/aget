# D281: Page Metadata Propagation

Decision: `aget` now carries generic page metadata from extraction into the public success shape, JSON envelopes, and run metadata artifacts.

Source inspection:

- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py` extracts page metadata before content filtering and HTML cleanup.
- `references/repos/crawl4ai/crawl4ai/utils.py` records `<title>`, description, keywords, author, Open Graph, Twitter, and article-prefixed meta tags, with title falling back to `og:title` or `twitter:title` when `<title>` is missing.
- `references/repos/crawl4ai/crawl4ai/models.py` includes `metadata` on `CrawlResult`, separate from content, media, links, and status fields.

Implementation boundary:

- Owned extraction reads generic `<head>` metadata before cleanup removes `title`, `meta`, or `link` elements.
- The public field is named `page_metadata` to avoid confusion with the run metadata artifact path.
- The compatibility Crawl4AI helper forwards `result.metadata` into the same field when available.
- Browser fallback and direct current-tab paths currently return an empty metadata map unless a later browser-specific slice extracts page metadata there.

Validation:

- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test --test mock_site_cli default_cli_fetch_includes_page_metadata_in_json_envelope`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test`
- `git diff --check`
