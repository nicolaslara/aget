# D330: Crawl4AI Unicode-Snob Markdown Option

The owned extractor now supports `crawl4ai.unicode_snob` for explicit Crawl4AI-style entity/Unicode replacement behavior.

Source check:

- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py` passes markdown-generator options into `CustomHTML2Text.update_params`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py` stores `unicode_snob` from config and uses it in `charref` and `entityref`.
- `references/repos/crawl4ai/crawl4ai/html2text/config.py` sets `UNICODE_SNOB = False` and defines ASCII-ish replacements for common entities such as copyright, em dash, curly quotes, arrows, middot, ligatures, and accented Latin vowels.

Implementation boundary:

- Owned extraction validates and accepts boolean `crawl4ai.unicode_snob`.
- Existing owned default Unicode markdown is preserved unless the option is supplied explicitly.
- `crawl4ai.unicode_snob=false` applies Crawl4AI-style replacement to decoded characters in owned markdown text.
- `crawl4ai.unicode_snob=true` preserves the current Unicode output.
- Because the owned HTML parser decodes entity references before markdown rendering, the owned option cannot distinguish source entities from literal Unicode characters. The explicit `false` mode intentionally normalizes the decoded characters either way.
- The Crawl4AI command helper, mock-backend allowlist, README, OpenCode tool text, unsupported-option error text, and command-option validation are aligned.

Validation:

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options`
- `python3 -m py_compile scripts/aget_crawl4ai_compat/options.py`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

Confidence: Medium. The parser-distinction caveat is explicit; option parsing, default preservation, explicit ASCII replacement behavior, command-helper forwarding, and option allowlists are covered by focused tests and the standard gate.
