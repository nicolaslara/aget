# D307: Crawl4AI `default_image_alt` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/config.py`: `DEFAULT_IMAGE_ALT` defaults to an empty string.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py`: the html2text CLI exposes `--default-image-alt` and assigns `h.default_image_alt`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: image handling computes `alt = attrs.get("alt") or self.default_image_alt`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: the effective alt value is then reused by raw image HTML, image-to-alt output, and normal image markdown.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options into `CustomHTML2Text.update_params`; no active default override changes the empty default alt.

## Decision

Add owned backend support for `crawl4ai.default_image_alt` while preserving the empty default.

## Boundary

- Default owned markdown still renders missing/empty image alt text as an empty alt label.
- `crawl4ai.default_image_alt=<text>` applies when the source image lacks `alt` or has an empty `alt`.
- Existing non-empty source alt text remains stronger than the configured default.
- The effective alt is used for normal image markdown, `crawl4ai.images_to_alt=true`, and raw image HTML modes.
- `crawl4ai.ignore_images=true` remains a stronger image gate.
- Existing base-URL behavior remains for normal image markdown targets.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.default_image_alt`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
