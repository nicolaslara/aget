# D345: Google Doc markdown option

## Decision

Owned markdown now accepts `crawl4ai.google_doc` and, when explicitly enabled,
renders Google Docs-style inline CSS emphasis for bold, italic, and fixed-width
spans.

## Source Evidence

- Crawl4AI snapshot: `references/repos/crawl4ai` at
  `1debe5f5fcc118ced10826a1040a81f9b77e9255`.
- `crawl4ai/markdown_generation_strategy.py` forwards `options` into
  `CustomHTML2Text.update_params`, so `google_doc` is a supported markdown
  option even though it is not one of the default overrides.
- `crawl4ai/html2text/__init__.py` initializes `google_doc = False`, tracks
  parent style context when enabled, and calls `handle_emphasis` for non-heading
  elements.
- `handle_emphasis` maps Google-style `font-weight` values to strong markers,
  `font-style: italic` to emphasis markers, fixed-width fonts such as Consolas
  to inline code, and uses `hide_strikethrough` to suppress line-through text.
- Direct local source-snapshot execution of `CustomHTML2Text` showed
  `google_doc=true` renders styled bold/italic/fixed-width spans as markdown,
  while line-through text is emitted as plain text unless
  `hide_strikethrough=true`.

## Owned Boundary

- Default owned markdown remains unchanged: style attributes are still pruned
  and styled spans render as plain text.
- `crawl4ai.google_doc=true` preserves style attributes through owned cleanup so
  the markdown renderer can see Google Docs-style inline CSS.
- Owned support is deliberately scoped to inline CSS emphasis used by current
  callers: bold, italic, and fixed-width spans. It does not attempt full Google
  Docs export reconstruction, such as style-derived list nesting.
- The existing `crawl4ai.hide_strikethrough` boundary still removes
  line-through styled elements before markdown rendering.
- The Crawl4AI command compatibility helper, mock-backend validation, README,
  OpenCode tool text, and unsupported-option surface are aligned.

## Validation

- `cargo test aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
