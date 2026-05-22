# D303: Crawl4AI `use_automatic_links` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/config.py`: `USE_AUTOMATIC_LINKS` defaults to true.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: anchor start records `maybe_automatic_link` before normal link close handling.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: text matching the recorded absolute href emits `<url>` only when `self.use_automatic_links` is true; otherwise handling falls back to an inline markdown link.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py`: the html2text CLI exposes `--no-automatic-links`.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options into `CustomHTML2Text.update_params`; no active default override disables automatic links.

## Decision

Add owned backend support for `crawl4ai.use_automatic_links` while preserving default Crawl4AI-style automatic absolute-link rendering.

## Boundary

- Default owned markdown still renders same-text absolute HTTP(S) anchors as `<url>`.
- `crawl4ai.use_automatic_links=false` renders same-text absolute HTTP(S) anchors as explicit markdown links.
- Explicit markdown fallback preserves link titles.
- `crawl4ai.ignore_links`, `crawl4ai.skip_internal_links`, and `crawl4ai.ignore_mailto_links` remain stronger gates.
- `crawl4ai.protect_links=true` still wraps explicit markdown link targets when automatic links are not used.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.use_automatic_links`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
