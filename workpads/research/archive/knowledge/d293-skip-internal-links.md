# D293: Crawl4AI `skip_internal_links` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `CustomHTML2Text.__init__` sets `skip_internal_links = False`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: anchor handling skips pushing a link target only when `skip_internal_links` is enabled and the raw `href` starts with `#`.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options into `CustomHTML2Text.update_params`.

## Decision

Add owned backend support for `crawl4ai.skip_internal_links` while preserving the existing default fragment-link behavior.

## Boundary

- Default owned markdown still resolves fragment links such as `href="#details"` against the page/base URL.
- `crawl4ai.skip_internal_links=true` suppresses fragment-only markdown link targets but preserves the visible anchor text.
- This is a markdown-rendering option, not an HTML cleanup option: cleaned HTML and text extraction remain unchanged.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)` instead of `CrawlerRunConfig`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.skip_internal_links`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
