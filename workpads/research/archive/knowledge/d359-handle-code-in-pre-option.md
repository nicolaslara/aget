# D359: Handle Code In Pre Markdown Option

## Decision

Owned markdown extraction supports `crawl4ai.handle_code_in_pre` for callers that need Crawl4AI-style inline-code markers preserved inside fenced `<pre>` blocks.

## Source Evidence

- `references/repos/crawl4ai`, commit `1debe5f5fcc118ced10826a1040a81f9b77e9255`, `crawl4ai/html2text/__init__.py`.
- `CustomHTML2Text.__init__` defines `handle_code_in_pre` with default `False`.
- `CustomHTML2Text.update_params` accepts `handle_code_in_pre`.
- `CustomHTML2Text.handle_tag` ignores `<code>` tags inside `<pre>` by default, but emits backticks around those tags when `handle_code_in_pre` is true.
- `CustomHTML2Text.handle_data` still emits raw preformatted text while inside `<pre>`.

## Implementation

- `crawl4ai.handle_code_in_pre` is parsed and validated as a boolean owned extractor option.
- The default owned markdown output for fenced code blocks is unchanged.
- When enabled, owned markdown preserves raw preformatted text while wrapping `<code>` descendants inside `<pre>` in backticks.
- The Crawl4AI command compatibility helper, mock-backend validation, README, OpenCode tool text, and unsupported-option error text include the new option.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.

## Validation

- `cargo fmt --check`
- `cargo test aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py scripts/crawl4ai_extract.py`
- `git diff --check`
- `cargo test`

## Follow-Up

- I19d remains open for fuller Crawl4AI-quality markdown/readability and richer rendered-page readiness.
