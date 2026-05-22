# D309: Crawl4AI Quote Marker Markdown Options

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/config.py`: `OPEN_QUOTE` and `CLOSE_QUOTE` both default to `"`.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py`: the html2text CLI exposes `--open-quote` and `--close-quote`, then assigns `h.open_quote` and `h.close_quote`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `<q>` handling emits the configured opening quote on entry and closing quote on exit.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options through `CustomHTML2Text.update_params`.

## Decision

Add owned backend support for `crawl4ai.open_quote` and `crawl4ai.close_quote` while preserving the double-quote defaults.

## Boundary

- Default owned markdown still renders `<q>quoted</q>` as `"quoted"`.
- `crawl4ai.open_quote=<text>` and `crawl4ai.close_quote=<text>` customize the markers used around `<q>` content.
- Existing inline markdown normalization, emphasis, inline code, and link handling remain unchanged.
- `crawl4ai.only_text=true` remains a stronger text-only path for eligible inline tags and returns raw text rather than quote markers.
- The Crawl4AI command compatibility helper routes both options through `DefaultMarkdownGenerator(options=...)`.

## Safety And Compatibility Notes

- The options do not execute user JavaScript and do not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include both quote options.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
