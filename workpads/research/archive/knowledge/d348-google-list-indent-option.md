# D348: Google List Indent Option

## Decision

`aget` supports `crawl4ai.google_list_indent` for Google Docs-style markdown lists when `crawl4ai.google_doc=true`.

## Source Evidence

- `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`.
- `crawl4ai/html2text/config.py` defaults `GOOGLE_LIST_INDENT = 36`.
- `crawl4ai/html2text/__init__.py` stores `google_list_indent`, uses `google_list_style(tag_style)` for Google Docs list type, and prefixes `li` output with two spaces per `google_nest_count(tag_style)`.
- `google_nest_count` divides `li` `margin-left` pixels by `google_list_indent`.
- `crawl4ai/html2text/utils.py` maps `list-style-type` values `disc`, `circle`, `square`, and `none` to unordered lists; other explicit values are ordered.

## Implementation

- Added owned backend option parsing and validation for `crawl4ai.google_list_indent`, defaulting to 36.
- Kept default owned list output unchanged unless `crawl4ai.google_doc=true`.
- When `google_doc=true`, the owned markdown renderer:
  - infers ordered vs unordered list markers from list `style="list-style-type:..."`;
  - infers `li` indentation from `margin-left / google_list_indent`;
  - keeps existing list rendering for normal HTML lists.
- Updated the Crawl4AI compatibility helper, mock backend option validation, README, and OpenCode tool text.

## Validation

- `cargo fmt --check`
- `cargo test aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test get_command_backend_accepts_scan_full_page_options`

## Follow-Up

- I19d remains open for broader Crawl4AI-quality readability/markdown and still-richer rendered-page readiness.
