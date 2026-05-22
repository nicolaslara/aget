# D294: Crawl4AI `include_sup_sub` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `CustomHTML2Text.__init__` sets `include_sup_sub = False`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `HTML2Text.handle_tag` emits literal `<sup>`/`</sup>` and `<sub>`/`</sub>` tags only when `include_sup_sub` is enabled.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options into `CustomHTML2Text.update_params`.

## Decision

Add owned backend support for `crawl4ai.include_sup_sub` while preserving the default plain-text rendering of `sup` and `sub` content.

## Boundary

- Default owned markdown still renders `sup` and `sub` child text without literal HTML wrappers.
- `crawl4ai.include_sup_sub=true` preserves literal `<sup>` and `<sub>` wrappers around inline markdown content.
- This is a markdown-rendering option, not an HTML cleanup option: cleaned HTML and text extraction remain unchanged.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)` instead of `CrawlerRunConfig`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.include_sup_sub`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
