# D300: Crawl4AI `ignore_mailto_links` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: base `HTML2Text` reads `ignore_mailto_links` from config, and active `CustomHTML2Text` sets `ignore_mailto_links = True`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: anchor handling skips `mailto:` hrefs only when `self.ignore_mailto_links` is true.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py`: the html2text CLI exposes `--ignore-mailto-links`.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options into `CustomHTML2Text.update_params`.

## Decision

Add owned backend support for `crawl4ai.ignore_mailto_links` while preserving the active Crawl4AI markdown default of suppressing mailto links.

## Boundary

- Default owned markdown still renders mailto anchors as visible child text without markdown link targets.
- `crawl4ai.ignore_mailto_links=false` renders mailto anchors as normal markdown links.
- `crawl4ai.ignore_links=true` remains stronger and suppresses all anchor link targets, including mailto.
- `crawl4ai.skip_internal_links` remains fragment-only and does not affect mailto links.
- This is a markdown-rendering option, not HTML cleanup.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Mailto link rendering can expose email addresses already present in page content; it is opt-in and default suppression remains unchanged.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.ignore_mailto_links`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
