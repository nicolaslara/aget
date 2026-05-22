# D328: Crawl4AI Single-Line-Break Markdown Option

The owned extractor now supports `crawl4ai.single_line_break` for opt-in Crawl4AI markdown compatibility:

- Owned extraction validates the option as a boolean and accepts both `true` and `false`.
- `crawl4ai.single_line_break=true` collapses non-code blank lines after owned markdown normalization.
- Fenced code block contents keep their internal blank lines.
- Existing owned default markdown shape is preserved unless the option is explicitly supplied. This keeps the current `aget` output contract stable while allowing callers to request the Crawl4AI-style compact line-break mode.
- The Crawl4AI compatibility command helper, mock backend allowlist, README, OpenCode tool schema, and unsupported-option error text are aligned with the accepted option.

Source check:

- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` includes `single_line_break: True` in `DefaultMarkdownGenerator` default options.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` maps `single_line_break` into `p()` paragraph spacing.
- Direct source-snapshot execution with `CustomHTML2Text` showed `single_line_break=true` collapses adjacent paragraph gaps to single newlines, while `false` keeps blank lines between paragraphs.

Default boundary:

- Crawl4AI's `DefaultMarkdownGenerator` default is `true`.
- The owned extractor keeps its established default blank-line markdown formatting for now and exposes the compact mode as an explicit compatibility option.
- This leaves a known default-output parity question for final I19d/I19h review instead of silently changing broad markdown output in a small option slice.

Validation:

- `cargo fmt --check`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `cargo test`
- `git diff --check`

Confidence: Medium. The option parser, compact output behavior, code-fence preservation, command-helper forwarding, and option allowlists are covered. The remaining risk is the deliberately recorded default-output parity boundary.
