# D295: Crawl4AI `ignore_links` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `CustomHTML2Text.__init__` sets `ignore_links = False`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: anchor handling runs only when `tag == "a" and not self.ignore_links`, so enabled `ignore_links` leaves anchor child content visible without emitting markdown link targets.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` defaults `ignore_links` to `False` and forwards markdown/html2text options into `CustomHTML2Text.update_params`.

## Decision

Add owned backend support for `crawl4ai.ignore_links` while preserving the existing default link markdown behavior.

## Boundary

- Default owned markdown still renders anchors with markdown link targets, link titles, linked headings, automatic absolute links, empty labels, and linked images.
- `crawl4ai.ignore_links=true` renders anchor children as normal visible content without markdown link targets.
- Child images inside ignored anchors still render as images because the option suppresses anchor handling, not image handling.
- This is a markdown-rendering option, not an HTML cleanup option: cleaned HTML and text extraction remain unchanged.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)` instead of `CrawlerRunConfig`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.ignore_links`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
