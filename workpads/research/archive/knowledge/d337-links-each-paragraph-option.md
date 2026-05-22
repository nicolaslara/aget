# D337: Crawl4AI Links-Each-Paragraph Markdown Option

The owned extractor now supports `crawl4ai.links_each_paragraph` for paragraph-scoped reference-link definitions.

Source check:

- `references/repos/crawl4ai/crawl4ai/html2text/config.py` defaults `LINKS_EACH_PARAGRAPH` to false.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py` exposes `--links-after-para` by setting `links_each_paragraph=true`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` flushes pending reference links when a paragraph break is emitted and `links_each_paragraph` is true, otherwise it flushes at end of document.
- Flushed links are removed from the pending list, so a repeated link in a later paragraph gets a new reference number.

Implementation boundary:

- Owned extraction validates and accepts boolean `crawl4ai.links_each_paragraph`.
- Existing/default owned reference-link output remains unchanged because `links_each_paragraph` defaults to false.
- When `crawl4ai.inline_links=false` and `crawl4ai.links_each_paragraph=true`, owned markdown emits pending reference definitions after paragraph blocks and gives repeated links in later paragraphs fresh reference numbers.
- Existing owned automatic absolute-link behavior remains stronger than reference-link conversion, so matching absolute URL labels still render as `<https://...>` when `crawl4ai.use_automatic_links=true`.
- Existing ignored/skipped link options remain stronger than `inline_links=false` and `links_each_paragraph=true`.
- The Crawl4AI command helper, mock-backend allowlist, README, OpenCode tool text, unsupported-option error text, and command-option validation are aligned.

Validation:

- `cargo fmt`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

Confidence: High. The source behavior is narrow, default output remains unchanged, and opt-in paragraph-scoped reference flushing is covered by focused markdown output plus command-option validation.
