# D298: Crawl4AI `protect_links` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` defaults `protect_links` to `False` and forwards markdown/html2text options into `CustomHTML2Text.update_params`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: anchor handling wraps `href` values in `<...>` when `self.protect_links` is enabled, then emits the normal inline link target path.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: automatic links whose label equals an absolute URL still render as `<url>` through the existing automatic-link branch.

## Decision

Add owned backend support for `crawl4ai.protect_links` while preserving the existing default markdown link target behavior.

## Boundary

- Default owned markdown still escapes link targets with the existing target escaping.
- `crawl4ai.protect_links=true` wraps anchor link targets in angle brackets.
- Automatic absolute links that already render as `<url>` are unchanged.
- Image `src` markdown targets are unchanged; this option applies to anchor `href` targets, including linked-image anchor targets.
- This is a markdown-rendering option, not an HTML cleanup option.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)` instead of `CrawlerRunConfig`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.protect_links`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
