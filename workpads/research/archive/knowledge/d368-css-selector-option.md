# D368: CSS Selector Option

## Decision

Owned extraction now accepts Crawl4AI's `crawl4ai.css_selector` backend option as a compatibility selector for content extraction.

## Source Evidence

- `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`.
- `crawl4ai/async_configs.py` defines `CrawlerRunConfig.css_selector` as the CSS selector used to extract a specific page portion.
- `crawl4ai/content_scraping_strategy.py` selects matching elements into a wrapper and falls back to the body on selector misses or selector errors in the static scraping path.
- `crawl4ai/async_crawler_strategy.py` serializes `querySelectorAll` matches when browser rendering captures HTML with `css_selector` set.

## Implementation

- `OwnedExtractorOptions` validates and stores `crawl4ai.css_selector` as an optional selector string.
- Top-level `--selector` / API selector continues to take precedence over the backend option.
- `crawl4ai.css_selector` takes precedence over browser-fallback's internal `body` fallback selector.
- Selector misses and invalid selectors keep the existing owned fallback-to-root behavior, matching Crawl4AI's static scraper behavior.
- `crawl4ai.target_elements` remains scoped under the selected roots when both options are supplied.
- The command mock, README, and OpenCode tool text list the new supported option.
- The Crawl4AI compatibility helper now accepts `css_selector` and also includes the recently added `excluded_selector` and `remove_consent_popups` option keys so helper validation stays aligned with the owned/mock option surface.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.

## Validation

- `cargo test aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/crawl4ai_extract.py scripts/aget_crawl4ai_compat/options.py`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

## Follow-Up

- I19d remains open for fuller Crawl4AI-quality readability/markdown and richer rendered-page readiness.
