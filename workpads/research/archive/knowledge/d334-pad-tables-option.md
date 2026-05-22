# D334: Crawl4AI Pad-Tables Markdown Option

The owned extractor now supports `crawl4ai.pad_tables` for opt-in markdown table column padding.

Source check:

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` calls `pad_tables_in_text(markdown)` after `optwrap` when `pad_tables` is true.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` emits table marker blocks and simpler inter-cell delimiters when `pad_tables` is active.
- `references/repos/crawl4ai/crawl4ai/html2text/utils.py` reformats buffered table lines by computing max column widths and padding cells with a one-character right margin.

Implementation boundary:

- Owned extraction validates and accepts boolean `crawl4ai.pad_tables`.
- Existing/default owned table markdown is preserved because `pad_tables` defaults to false.
- When `crawl4ai.pad_tables=true`, owned markdown post-processes GFM table blocks and pads each column to the widest cell in that table.
- This slice applies the Crawl4AI table-padding concept to the owned renderer's existing GFM table output instead of introducing Crawl4AI's internal table marker representation.
- The Crawl4AI command helper, mock-backend allowlist, README, OpenCode tool text, unsupported-option error text, and command-option validation are aligned.

Validation:

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

Confidence: High. The source behavior is narrow, default output remains unchanged, and opt-in padding is covered by focused table markdown output plus command-option validation.
