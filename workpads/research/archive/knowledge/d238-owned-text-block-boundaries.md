# D238: Owned Text Block Boundaries

## Source Evidence

- `scripts/crawl4ai_extract.py`: the compatibility text path uses `TextExtractor` to add line breaks around common block tags (`main`, `article`, `section`, `div`, headings, paragraphs, list items, table rows, and `br`) and then normalizes whitespace while dropping blank lines.
- `references/repos/crawl4ai/crawl4ai/content_scraping_strategy.py`: Crawl4AI generates cleaned HTML before callers select text-like outputs, so the aget compatibility helper's text conversion boundary is the behavior existing `aget` callers saw for command-backed `--content-format text`.

## Decision

- Add an owned text renderer under `src/extraction/owned/text.rs` instead of using descendant-text flattening.
- Preserve markdown and cleaned-HTML rendering paths.
- Use the owned text renderer for both `OutputFormat::Text` and the `"content"` field in owned JSON output, preserving the current JSON shape while improving the text payload's block boundaries.
- Join multiple selected roots or `crawl4ai.target_elements` with a single newline.

## Validation

- `cargo fmt --check`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice -- --nocapture`
- `git diff --check`
- `cargo test`
