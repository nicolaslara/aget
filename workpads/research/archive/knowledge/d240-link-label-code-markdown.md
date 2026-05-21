# D240: Link-Label Code Markdown Boundary

## Source Evidence

- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `HTML2Text` tracks `inside_link` while rendering anchors, and `CustomHTML2Text.handle_tag` emits code backticks only when not inside a link.
- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: the default markdown generator uses `CustomHTML2Text` with `mark_code=true`, so standalone inline code remains marked while code inside anchors follows the link boundary.

## Decision

- Keep owned standalone inline `code`, `kbd`, and `tt` rendering as backtick-marked markdown.
- When rendering anchor labels, treat inline `code`, `kbd`, and `tt` text as plain label text so generated links match Crawl4AI's link-label behavior.
- Preserve existing link target, title, automatic absolute-link, and linked-image behavior.

## Validation

- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`
