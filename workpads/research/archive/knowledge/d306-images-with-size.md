# D306: Crawl4AI `images_with_size` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/config.py`: `IMAGES_WITH_SIZE` defaults to false.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py`: the html2text CLI exposes `--images-with-size` and assigns `h.images_with_size`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: image handling emits raw `<img ... />` HTML when `images_as_html` is true or when `images_with_size` is true and the image has `width` or `height`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: raw image HTML preserves `src`, optional `width`, optional `height`, and optional `alt`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: the raw image branch runs before normal image markdown and `images_to_alt` handling.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options into `CustomHTML2Text.update_params`; no active default override enables size-triggered raw image HTML.

## Decision

Add owned backend support for `crawl4ai.images_with_size` while preserving default markdown image rendering.

## Boundary

- Default owned markdown still renders images as `![alt](src)`.
- `crawl4ai.images_with_size=true` renders raw `<img src='...' ... />` tags only for images with `width` or `height`.
- Unsized images remain normal markdown images unless another image option changes them.
- Raw image HTML preserves `width`, `height`, and `alt` attributes when present.
- `crawl4ai.images_as_html=true` remains the broader raw-image mode for all images.
- `crawl4ai.images_with_size=true` takes precedence over `crawl4ai.images_to_alt=true` for sized images, matching the source branch order.
- `crawl4ai.ignore_images=true` remains a stronger image gate.
- Linked images keep the existing owned link wrapping behavior around the rendered image representation.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.images_with_size`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
