# D336: Crawl4AI Inline-Links Markdown Option

The owned extractor now supports `crawl4ai.inline_links` for opt-in reference-style markdown links.

Source check:

- `references/repos/crawl4ai/crawl4ai/html2text/config.py` defaults `INLINE_LINKS` to true.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py` exposes `--reference-links` by setting `inline_links=false`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` emits normal inline links when `inline_links` is true.
- When `inline_links` is false, `CustomHTML2Text` reuses link numbers by matching `href` plus optional `title`, writes `[label][n]`, and later appends `   [n]: url` definitions.

Implementation boundary:

- Owned extraction validates and accepts boolean `crawl4ai.inline_links`.
- Existing/default owned markdown links remain unchanged because `inline_links` defaults to true.
- When `crawl4ai.inline_links=false`, owned markdown emits reference-style links and image references, reuses deterministic numbers for matching resolved URL plus title, and appends reference definitions.
- Existing owned automatic absolute-link behavior remains stronger than reference-link conversion, so `https://...` labels still render as `<https://...>` when `crawl4ai.use_automatic_links=true`.
- Existing ignored/skipped link options remain stronger than `inline_links=false`.
- The Crawl4AI command helper, mock-backend allowlist, README, OpenCode tool text, unsupported-option error text, and command-option validation are aligned.

Validation:

- `cargo fmt`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

Confidence: High. The source behavior is narrow, default output remains unchanged, and opt-in reference-link rendering is covered by focused markdown output plus command-option validation.
