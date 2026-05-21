# D194: Markdown Writer and Inline Helper Split

The markdown renderer was mechanically reduced to support continued Crawl4AI-quality markdown work without loading all renderer state, inline handling, table handling, and normalization in one file.

Resulting boundaries:

- `src/extraction/markdown/mod.rs`: renderer dispatch plus block, list, definition-list, code-block, and blockquote rendering.
- `src/extraction/markdown/writer.rs`: `MarkdownWriter` state, URL resolution, inline text pushing, blank-line management, and abbreviation-definition accumulation.
- `src/extraction/markdown/inline.rs`: link, image, abbreviation, inline-child markdown, and raw-text helpers.
- `src/extraction/markdown/table.rs`: table rendering.
- `src/extraction/markdown/normalize.rs`: markdown escaping and normalization helpers.

The split did not intentionally change markdown output, table output, link/image resolution, abbreviation output, or only-text behavior.

Validation:

- `cargo fmt --check`
- `cargo test --test mock_site_cli aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test aget_extractor::tests`
- `cargo test`
- `git diff --check`
