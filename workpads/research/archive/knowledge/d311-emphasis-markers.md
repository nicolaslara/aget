# D311: Crawl4AI Emphasis Marker Markdown Options

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `HTML2Text.__init__` sets `self.emphasis_mark = "_"` and `self.strong_mark = "**"`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `em`/`i`/`u` and `strong`/`b` output use those marker strings when `ignore_emphasis` is false.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: Google's style-emphasis path also writes `self.emphasis_mark` and `self.strong_mark` for italic and bold spans.
- `references/repos/crawl4ai/crawl4ai/html2text/cli.py`: `--asterisk-emphasis` sets `h.emphasis_mark = "*"` and `h.strong_mark = "__"`.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` forwards markdown/html2text options through `CustomHTML2Text.update_params`.

## Decision

Add owned backend support for `crawl4ai.emphasis_mark` and `crawl4ai.strong_mark` while preserving the `_` and `**` defaults.

## Boundary

- Default owned markdown still renders `em`/`i`/`u` with `_` and `strong`/`b` with `**`.
- `crawl4ai.emphasis_mark=<text>` changes only emphasis wrappers.
- `crawl4ai.strong_mark=<text>` changes only strong wrappers.
- `crawl4ai.ignore_emphasis=true` remains stronger and renders emphasis/strong children as visible plain content without markers.
- Inline code, quote markers, link-label rendering, abbreviation definitions, and markdown normalization remain unchanged.
- The Crawl4AI command compatibility helper routes both options through `DefaultMarkdownGenerator(options=...)`.

## Safety And Compatibility Notes

- The options do not execute user JavaScript and do not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.emphasis_mark` and `crawl4ai.strong_mark`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo fmt --check`
- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo test`
- `git diff --check`
