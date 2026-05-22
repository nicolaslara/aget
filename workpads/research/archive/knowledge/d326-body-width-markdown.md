# D326: Crawl4AI Body-Width Markdown Option

The owned extractor now supports `crawl4ai.body_width` for Crawl4AI markdown compatibility:

- The default remains `0`, which keeps owned markdown unwrapped and preserves existing output.
- Positive values wrap plain markdown paragraph lines on whitespace after normalization.
- Markdown structure lines are preserved instead of wrapped: headings, lists, tables, blockquotes, fenced code, indented definitions, image lines, abbreviation definitions, hard-break lines, and horizontal rules.
- Invalid owned values fail before extraction with a typed `crawl4ai.body_width expects a non-negative integer value` error.
- The Crawl4AI compatibility command helper, mock backend allowlist, README, OpenCode tool schema, and unsupported-option error text are aligned with the new option.

Source check:

- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` shows `DefaultMarkdownGenerator` defaulting `body_width` to `0`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` treats falsey `body_width` as no wrapping and applies wrapping only when a width is set.
- The owned implementation copies the behavior boundary and option default, not upstream code.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo test`
- `git diff --check`

Confidence: Medium-high. Deterministic coverage proves the option parser, plain paragraph wrapping, structure preservation, command-adapter forwarding, and option allowlists.
