# D296: Crawl4AI `ignore_images` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: image tag handling runs only when `tag == "img" and start and not self.ignore_images`.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` defaults `ignore_images` to `False` and forwards markdown/html2text options into `CustomHTML2Text.update_params`.

## Decision

Add owned backend support for `crawl4ai.ignore_images` while preserving the existing default image markdown behavior.

## Boundary

- Default owned markdown still renders standalone images and linked images.
- `crawl4ai.ignore_images=true` suppresses image markdown.
- Linked image anchors still preserve their enclosing anchor behavior; only the image label is suppressed.
- This is a markdown-rendering option, not an HTML cleanup option: cleaned HTML and text extraction remain unchanged. Use cleanup options such as `crawl4ai.exclude_all_images` when the desired behavior is HTML image removal.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)` instead of `CrawlerRunConfig`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.ignore_images`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
