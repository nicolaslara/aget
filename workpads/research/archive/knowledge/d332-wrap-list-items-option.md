# D332: Crawl4AI Wrap-List-Items Markdown Option

The owned extractor now supports `crawl4ai.wrap_list_items` for body-width list-item wrapping compatibility.

Source check:

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` stores `wrap_list_items` from config and passes it into `skipwrap` during `optwrap`.
- `references/repos/crawl4ai/crawl4ai/html2text/utils.py` preserves paragraphs whose stripped text starts with `-` or `*` when `wrap_list_items` is false.
- `references/repos/crawl4ai/crawl4ai/html2text/config.py` defaults `WRAP_LIST_ITEMS = False`, so list items are not wrapped unless the caller opts in.

Implementation boundary:

- Owned extraction validates and accepts boolean `crawl4ai.wrap_list_items`.
- Existing/default owned body-width behavior is preserved because `wrap_list_items` defaults to false.
- When `crawl4ai.body_width` is positive and `crawl4ai.wrap_list_items=true`, owned body-width wrapping may wrap top-level markdown list item lines.
- Indented markdown lines remain preserved by the owned renderer's broader indentation-preservation rule; this slice covers the current owned top-level list output used by the markdown renderer.
- The Crawl4AI command helper, mock-backend allowlist, README, OpenCode tool text, unsupported-option error text, and command-option validation are aligned.

Validation:

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

Confidence: High. The source behavior is narrow, default output remains unchanged, and opt-in wrapping is covered by focused body-width markdown output plus command-option validation.
