# D267: Crawl4AI Linked-Heading Markdown

## Source Inspection

- `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`: the active default markdown generator uses `CustomHTML2Text` with raw markdown output.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: the base `HTML2Text.handle_tag` path has explicit logic for heading tags encountered inside an active anchor stack. It moves the heading marker outside the link so linked headings render as headings whose text is linked, rather than links whose label contains a literal Markdown heading marker.
- `references/repos/crawl4ai/crawl4ai/html2text/__init__.py`: `CustomHTML2Text` still delegates normal non-code, non-preserved tags to the base handler, so the linked-heading behavior remains active in Crawl4AI's default raw markdown path.

## Decision

Render owned markdown for anchors whose only non-whitespace child is an `h1`-`h6` as linked headings:

```markdown
## [Linked Heading](https://example.test/linked-heading "Heading title")
```

Normal anchors, linked images, standalone headings, base URL resolution, link titles, and linked inline-code label handling remain unchanged.

## Validation

- `cargo test --test mock_site_cli aget_extractor::aget_extractor_backend_covers_static_http_parity_slice`
- `cargo fmt --check && cargo test && git diff --check`
