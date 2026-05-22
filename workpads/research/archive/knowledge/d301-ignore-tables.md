# D301: Crawl4AI `ignore_tables` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: table handling checks `self.ignore_tables` for `table`, `tr`, `td`, and `th` tags.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: when `ignore_tables` is true, table/cell tags emit no markdown table syntax and `tr` end tags emit a soft row break.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py`: the html2text CLI exposes `--ignore-tables` as "Ignore table-related tags (table, th, td, tr) while keeping rows."
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options into `CustomHTML2Text.update_params`; no active default override sets `ignore_tables`, so the markdown default remains false.

## Decision

Add owned backend support for `crawl4ai.ignore_tables` while preserving default markdown table rendering.

## Boundary

- Default owned markdown still renders supported HTML tables as GitHub-flavored markdown tables.
- `crawl4ai.ignore_tables=true` renders caption and row/cell content as plain markdown lines without table pipes or separator syntax.
- Link, image, emphasis, mailto, and other inline markdown options still apply inside ignored table cells.
- This is a markdown-rendering option, not cleaned-HTML table removal.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.ignore_tables`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
