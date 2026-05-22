# D338: Crawl4AI line-start escape markdown options

## Decision

`AgetExtractor` now supports `crawl4ai.escape_dot`, `crawl4ai.escape_plus`, and `crawl4ai.escape_dash` for owned markdown rendering and compatibility command validation.

## Source Evidence

- Crawl4AI snapshot: `references/repos/crawl4ai` at `1debe5f5fcc118ced10826a1040a81f9b77e9255`.
- `crawl4ai/html2text/utils.py` defines `escape_md_section(...)` with separate `escape_dot`, `escape_plus`, and `escape_dash` flags. These flags escape line-start ordered-list dots, plus bullets, and dash bullets.
- `crawl4ai/html2text/__init__.py` calls `escape_md_section(...)` for normal text outside code/pre blocks, using the instance flags.
- `crawl4ai/html2text/__init__.py` `CustomHTML2Text` defaults `escape_dot`, `escape_plus`, and `escape_dash` to `False`.
- `crawl4ai/markdown_generation_strategy.py` forwards `DefaultMarkdownGenerator(options=...)` into `CustomHTML2Text.update_params(...)`.

## Owned Boundary

- Existing owned defaults still escape accidental line-start ordered-list, plus-bullet, and dash-bullet markers. This preserves current stable markdown output and avoids turning ordinary text into generated Markdown lists.
- Explicit `crawl4ai.escape_dot=false`, `crawl4ai.escape_plus=false`, or `crawl4ai.escape_dash=false` disables the corresponding line-start escape for normal text.
- The option scope is normal text line-start escaping only. Structural `<ol>`, `<ul>`, and `<li>` rendering still emits real Markdown list markers.
- `crawl4ai.escape_snob=true` still performs broad Markdown character escaping before the line-start controls run.

## Validation

- `cargo fmt --check`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `cargo test`
- `git diff --check`
