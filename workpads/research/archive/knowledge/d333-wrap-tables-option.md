# D333: Crawl4AI Wrap-Tables Markdown Option

The owned extractor now supports `crawl4ai.wrap_tables` for body-width table-line wrapping compatibility.

Source check:

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` stores `wrap_tables` from config and passes it into `skipwrap` during `optwrap`.
- `references/repos/crawl4ai/crawl4ai/html2text/utils.py` preserves paragraphs matching the table regex when `wrap_tables` is false.
- `references/repos/crawl4ai/crawl4ai/html2text/config.py` defaults `WRAP_TABLES = False`, so table-like lines are not wrapped unless the caller opts in.

Implementation boundary:

- Owned extraction validates and accepts boolean `crawl4ai.wrap_tables`.
- Existing/default owned body-width behavior is preserved because `wrap_tables` defaults to false.
- When `crawl4ai.body_width` is positive and `crawl4ai.wrap_tables=true`, owned body-width wrapping may wrap markdown table lines.
- This option can intentionally produce broken markdown table syntax when a caller opts in, matching the source behavior that treats this as a body-width wrapping control rather than a table reflow feature.
- The Crawl4AI command helper, mock-backend allowlist, README, OpenCode tool text, unsupported-option error text, and command-option validation are aligned.

Validation:

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

Confidence: High. The source behavior is narrow, default output remains unchanged, and opt-in wrapping is covered by focused table markdown output plus command-option validation.
