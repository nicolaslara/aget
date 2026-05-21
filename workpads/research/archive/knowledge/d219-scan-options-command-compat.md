# D219: Scan Options Command Compatibility

Date: 2026-05-21

Source inspected:

- `scripts/crawl4ai_extract.py`: explicit Crawl4AI compatibility helper option parsing and validation.
- `references/repos/crawl4ai/crawl4ai/async_configs.py`: `scan_full_page`, `scroll_delay`, and `max_scroll_steps` are `CrawlerRunConfig` page-interaction options.
- `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py`: full-page scanning is a rendered-page readiness step and remains non-fatal on timeout/failure.

Decision:

- Keep the explicit Crawl4AI command helper's option allowlist aligned with the owned backend after D218.
- Add `crawl4ai.scan_full_page`, `crawl4ai.scroll_delay`, and `crawl4ai.max_scroll_steps` to `scripts/crawl4ai_extract.py`.
- Preserve the helper's existing safety boundary: JavaScript waits are still rejected before Crawl4AI import, and unknown options still fail explicitly.

Boundary:

- This does not make Crawl4AI a default runtime dependency again.
- This only keeps explicitly configured compatibility mode from rejecting options that the owned backend already supports.

Validation:

- `python3` import/call smoke for `scripts/crawl4ai_extract.py::parse_extractor_options` with `crawl4ai.scan_full_page`, `crawl4ai.scroll_delay`, and `crawl4ai.max_scroll_steps`
- `cargo test --test get_cli get_command_backend_accepts_scan_full_page_options -- --nocapture`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
