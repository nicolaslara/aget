# D370: Crawl4AI `keep_attrs` Cleanup Option

Date: 2026-05-23

## Decision

Owned extraction accepts `crawl4ai.keep_attrs` as a comma-separated backend option that preserves explicitly named attributes during cleaned-HTML serialization.

This is compatibility support for Crawl4AI's exposed `CrawlerRunConfig.keep_attrs` surface, not a broader cleaned-HTML policy change. The existing default allowlist, `crawl4ai.keep_data_attributes`, and selector-before-pruning order stay unchanged.

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/async_configs.py` documents `keep_attrs` as a list of HTML attributes to keep, defaults it to `[]`, stores it on `CrawlerRunConfig`, and serializes it through `dump()`.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py` keeps only `IMPORTANT_ATTRS` plus optional `data-*` attributes in the current `remove_unwanted_attributes_fast` call path. The owned implementation therefore records the exposed config compatibility boundary explicitly.

## Implementation Notes

- `src/extraction/owned/options.rs` stores `keep_attrs`.
- `src/extraction/owned/options/apply.rs` parses `crawl4ai.keep_attrs` as a comma-separated list.
- `src/extraction/html_clean/attributes.rs` preserves names listed in `keep_attrs` while retaining the existing Crawl4AI important attributes and `keep_data_attributes` behavior.
- The optional Crawl4AI compatibility helper forwards `keep_attrs` to real Crawl4AI when the installed version accepts it.

## Validation

Passed:

- `cargo test aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/crawl4ai_extract.py scripts/aget_crawl4ai_compat/options.py`
- Python helper parse smoke for `crawl4ai.keep_attrs=aria-label,rel`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`
