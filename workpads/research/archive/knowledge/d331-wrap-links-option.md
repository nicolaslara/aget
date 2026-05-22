# D331: Crawl4AI Wrap-Links Markdown Option

The owned extractor now supports `crawl4ai.wrap_links` for body-width wrapping compatibility.

Source check:

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` stores `wrap_links` from config and uses it during `optwrap`.
- `references/repos/crawl4ai/crawl4ai/html2text/utils.py` has `skipwrap`; when `wrap_links` is false and a paragraph contains a markdown link/reference, wrapping is skipped.
- `references/repos/crawl4ai/crawl4ai/html2text/config.py` defaults `WRAP_LINKS = True`, so default line wrapping still wraps paragraphs containing links.

Implementation boundary:

- Owned extraction validates and accepts boolean `crawl4ai.wrap_links`.
- Existing/default owned body-width behavior is preserved because `wrap_links` defaults to true.
- When `crawl4ai.body_width` is positive and `crawl4ai.wrap_links=false`, owned body-width wrapping preserves markdown lines containing inline markdown links.
- This is scoped to the owned renderer's inline-link markdown output; owned extraction does not add Crawl4AI reference-link output in this slice.
- The Crawl4AI command helper, mock-backend allowlist, README, OpenCode tool text, unsupported-option error text, and command-option validation are aligned.

Validation:

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

Confidence: High. The source behavior is narrow, default output remains unchanged, and the explicit false mode is covered by a focused link-containing body-width fixture plus command-option validation.
