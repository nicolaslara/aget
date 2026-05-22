# D371: Crawl4AI `prettiify` Cleaned-HTML Option

Date: 2026-05-23

## Decision

Owned extraction accepts `crawl4ai.prettiify` as a boolean backend option and applies Crawl4AI-style fast HTML formatting only when the requested output format is HTML.

Markdown, text, and JSON output remain unchanged. This matches Crawl4AI's boundary: `AsyncWebCrawler` applies `fast_format_html(cleaned_html)` after scraping/extraction and before returning `CrawlResult.cleaned_html`.

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/async_configs.py` documents `prettiify`, defaults it to `False`, stores it on `CrawlerRunConfig`, and serializes it through `dump()`.
- `references/repos/crawl4ai/crawl4ai/async_webcrawler.py` applies `fast_format_html(cleaned_html)` only when `config.prettiify` is true.
- `references/repos/crawl4ai/crawl4ai/utils.py` implements `fast_format_html` with a simple tag/content splitter and two-space indentation.

## Implementation Notes

- `src/extraction/owned/options.rs` stores `prettiify`.
- `src/extraction/owned/options/apply.rs` validates `crawl4ai.prettiify` as a boolean.
- `src/extraction/owned/page.rs` formats only owned HTML output after content extraction.
- The optional Crawl4AI compatibility helper forwards `prettiify` to real Crawl4AI when the installed version accepts it.

## Validation

Passed:

- `cargo test aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/crawl4ai_extract.py scripts/aget_crawl4ai_compat/options.py`
- Python helper parse smoke for `crawl4ai.prettiify=true`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`
