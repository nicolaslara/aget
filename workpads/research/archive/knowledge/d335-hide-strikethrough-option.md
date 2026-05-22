# D335: Crawl4AI Hide-Strikethrough Markdown Option

The owned extractor now supports `crawl4ai.hide_strikethrough` for opt-in suppression of styled strikethrough text.

Source check:

- `references/repos/crawl4ai/crawl4ai/html2text/cli.py` exposes `--hide-strikethrough` with default false and notes it is only relevant with `--google-doc`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` stores `hide_strikethrough` on `HTML2Text` with default false.
- `handle_emphasis` suppresses output only when parsed style emphasis contains `line-through` and `hide_strikethrough` is true.
- Semantic `<del>`, `<strike>`, and `<s>` tags are handled separately and still emit `~~` markers.

Implementation boundary:

- Owned extraction validates and accepts boolean `crawl4ai.hide_strikethrough`.
- Existing/default owned semantic strikethrough markdown remains unchanged because the option defaults to false.
- When `crawl4ai.hide_strikethrough=true`, owned markdown suppresses elements whose inline style contains `line-through`.
- Semantic `<del>`, `<strike>`, and `<s>` tags still render as `~~...~~`, matching Crawl4AI's separate semantic-tag path.
- The Crawl4AI command helper, mock-backend allowlist, README, OpenCode tool text, unsupported-option error text, and command-option validation are aligned.

Validation:

- `cargo fmt --check`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `cargo test`
- `git diff --check`

Confidence: High. The source behavior is narrow, default output remains unchanged, and opt-in suppression is covered by focused markdown output plus command-option validation.
