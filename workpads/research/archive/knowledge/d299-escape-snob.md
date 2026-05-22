# D299: Crawl4AI `escape_snob` Markdown Option

Date: 2026-05-22

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: `DefaultMarkdownGenerator` defaults `escape_snob` to `False` and forwards markdown/html2text options into `CustomHTML2Text.update_params`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `CustomHTML2Text.handle_data` delegates normal text to the base handler, while preserving raw data for `pre` and inline-code paths.
- `references/repos/crawl4ai/crawl4ai/html2text/utils.py` and `config.py`: `escape_snob=True` applies the broader markdown character matcher for normal text: backtick, star, underscore, braces, brackets, parens, hash, and bang.

## Decision

Add owned backend support for `crawl4ai.escape_snob` while preserving the existing default text escaping.

## Boundary

- Default owned markdown text escaping is unchanged.
- `crawl4ai.escape_snob=true` escapes Crawl4AI's broader normal-text markdown character set.
- Generated markdown syntax for structural tags, emphasis, links, images, blockquotes, and code fences remains generated syntax.
- Inline code and fenced pre/code content stay on their existing code-specific path and are not broadly escaped.
- This is a markdown-rendering option, not an HTML cleanup option.
- The Crawl4AI command compatibility helper routes the option through `DefaultMarkdownGenerator(options=...)` instead of `CrawlerRunConfig`.

## Safety And Compatibility Notes

- The option does not execute user JavaScript and does not broaden authenticated extraction.
- Command compatibility validation, the checked-in mock backend, README, OpenCode tool text, and owned unsupported-option error text now include `crawl4ai.escape_snob`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test get_command_backend_accepts_scan_full_page_options --test get_cli`
- `cargo test get_real_helper_rejects_unsupported_extractor_option_before_crawl4ai_import --test get_cli`
- `python3 -m py_compile scripts/crawl4ai_extract.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
