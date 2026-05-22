# D305: Crawl4AI `images_as_html` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/config.py`: `IMAGES_AS_HTML` defaults to false.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py`: the html2text CLI exposes `--images-as-html` and assigns `h.images_as_html`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: image handling emits raw `<img ... />` when `images_as_html` is true.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: raw image HTML preserves `src`, optional `width`, optional `height`, and optional `alt`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: raw image HTML is emitted before the normal image markdown and `images_to_alt` branches.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options into `CustomHTML2Text.update_params`; no active default override enables raw image HTML.

## Decision

Add owned backend support for `crawl4ai.images_as_html` while preserving default markdown image rendering.

## Boundary

- Default owned markdown still renders images as `![alt](src)`.
- `crawl4ai.images_as_html=true` renders raw `<img src='...' ... />` tags.
- Raw image HTML preserves `width`, `height`, and `alt` attributes when present.
- Raw image HTML uses the element's source `src` attribute rather than base-URL-resolved markdown image targets.
- `crawl4ai.images_as_html=true` takes precedence over `crawl4ai.images_to_alt=true`, matching the source branch order.
- `crawl4ai.ignore_images=true` remains a stronger image gate.
- Linked images keep the existing owned link wrapping behavior around the rendered image representation.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.images_as_html`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
