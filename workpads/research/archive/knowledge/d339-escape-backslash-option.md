# D339: Crawl4AI escape_backslash markdown option

## Decision

`AgetExtractor` now supports `crawl4ai.escape_backslash` for owned markdown rendering and compatibility command validation.

## Source Evidence

- Crawl4AI snapshot: `references/repos/crawl4ai` at `1debe5f5fcc118ced10826a1040a81f9b77e9255`.
- `crawl4ai/html2text/utils.py` defines `escape_md_section(...)` with `escape_backslash` and applies it before broad/snob and line-start escaping.
- `crawl4ai/html2text/__init__.py` calls `escape_md_section(...)` for normal text outside code/pre blocks, using the instance `escape_backslash` flag.
- `crawl4ai/html2text/__init__.py` `CustomHTML2Text` defaults `escape_backslash` to `False`.
- `crawl4ai/markdown_generation_strategy.py` forwards `DefaultMarkdownGenerator(options=...)` into `CustomHTML2Text.update_params(...)`.

## Owned Boundary

- Existing owned defaults still escape literal backslashes before Markdown-sensitive characters. This preserves current stable markdown output where literal source backslashes remain visible instead of becoming Markdown escapes.
- Explicit `crawl4ai.escape_backslash=false` disables that extra owned backslash protection for normal text.
- The option scope is normal text. Code/pre content, line-start marker escaping, broad `escape_snob`, and link/image target escaping keep their existing behavior.

## Validation

- `cargo fmt --check`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `cargo test`
- `git diff --check`
