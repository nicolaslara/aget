# D297: Crawl4AI `ignore_emphasis` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `ignore_emphasis` gates emphasis and strong markdown marker output for `em`/`i`/`u` and `strong`/`b`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `del`/`strike`/`s` handling is separate from `ignore_emphasis`.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` defaults `ignore_emphasis` to `False` and forwards markdown/html2text options into `CustomHTML2Text.update_params`.

## Decision

Add owned backend support for `crawl4ai.ignore_emphasis` while preserving the existing default emphasis and strong markdown behavior.

## Boundary

- Default owned markdown still renders `strong`/`b` as `**...**` and `em`/`i`/`u` as `_..._`.
- `crawl4ai.ignore_emphasis=true` renders those children as normal visible content without emphasis markers.
- Strikethrough tags still render as `~~...~~`.
- This is a markdown-rendering option, not an HTML cleanup option.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)` instead of `CrawlerRunConfig`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.ignore_emphasis`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
