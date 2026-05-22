# D304: Crawl4AI `images_to_alt` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/config.py`: `IMAGES_TO_ALT` defaults to false.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py`: the html2text CLI exposes `--images-to-alt`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: image handling keeps the image `src` as link data only when `images_to_alt` is false.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: when `images_to_alt` is true, image output discards image markdown and emits escaped alt text instead.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: linked images still participate in link output, and same-text absolute URL alt values can use the automatic `<url>` path.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options into `CustomHTML2Text.update_params`; no active default override enables image-to-alt output.

## Decision

Add owned backend support for `crawl4ai.images_to_alt` while preserving default markdown image rendering.

## Boundary

- Default owned markdown still renders images as `![alt](src)`.
- `crawl4ai.images_to_alt=true` renders image alt text only and does not emit image `src` targets.
- Linked images become normal links whose label is the escaped image alt text.
- Existing automatic same-text absolute link rendering remains enabled on the same page.
- `crawl4ai.ignore_images=true` remains a stronger image gate.
- Existing base-URL behavior remains for normal image markdown and linked-image anchor targets.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.images_to_alt`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
