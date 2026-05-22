# D310: Crawl4AI Unordered List Marker Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `HTML2Text.__init__` sets `self.ul_item_mark = "*"`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: unordered list item rendering emits `self.ul_item_mark + " "`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `update_params` assigns arbitrary option keys onto the converter, so `DefaultMarkdownGenerator(options=...)` can set `ul_item_mark`.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py`: the CLI dash-list flag sets `h.ul_item_mark = "-"`.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options through `CustomHTML2Text.update_params`.

## Decision

Add owned backend support for `crawl4ai.ul_item_mark` while preserving the `*` default.

## Boundary

- Default owned markdown still renders unordered list items with `*`.
- `crawl4ai.ul_item_mark=<text>` changes only unordered-list item prefixes.
- Ordered-list numbering remains unchanged.
- Nested-list indentation, inline content rendering, links, images, and markdown normalization remain unchanged.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.ul_item_mark`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo fmt --check`
- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo test`
- `git diff --check`
