# D327: Crawl4AI Mark-Code Markdown Option

The owned extractor now accepts `crawl4ai.mark_code` for Crawl4AI markdown compatibility:

- The source-backed default is `true`.
- Owned extraction validates the option as a boolean and accepts both `true` and `false`.
- The option is a compatibility no-op in the owned renderer because Crawl4AI's active `CustomHTML2Text` path emits inline backticks and fenced code for both `mark_code=true` and `mark_code=false`.
- Existing owned inline-code and fenced-code markdown output is preserved.
- The Crawl4AI compatibility command helper, mock backend allowlist, README, OpenCode tool schema, and unsupported-option error text are aligned with the accepted option.

Source check:

- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` includes `mark_code: True` in `DefaultMarkdownGenerator` default options.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` accepts `mark_code`, but direct source-snapshot execution with `CustomHTML2Text` produced the same inline-code and fenced-code output for `mark_code=true` and `mark_code=false`.
- The owned implementation copies that behavior boundary and default, not upstream code.

Validation:

- `cargo fmt --check`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `cargo test`
- `git diff --check`

Confidence: Medium. The option surface and output boundary are deterministic, but the behavior is intentionally recorded as a source-backed no-op rather than a new renderer mode.
