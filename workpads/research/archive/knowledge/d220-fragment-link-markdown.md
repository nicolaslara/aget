# D220: Fragment Link Markdown

Date: 2026-05-21

Source inspected:

- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: the active default generator uses `CustomHTML2Text`.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `CustomHTML2Text.__init__` sets `skip_internal_links = False`; the base `HTML2Text` default of skipping internal links is not the active Crawl4AI markdown path.

Decision:

- Preserve fragment-only links in owned markdown by resolving them against the page/base URL.
- Keep `mailto:` suppression unchanged because `CustomHTML2Text.__init__` still sets `ignore_mailto_links = True`.
- Treat this as a correction to the earlier D113 interpretation, which followed the base `html2text` default instead of the active `CustomHTML2Text` configuration.

Boundary:

- This only affects markdown link rendering for `href="#..."`.
- It does not add JavaScript URL execution, link crawling, or site-specific anchor handling.

Validation:

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
