# Knowledge Archive D177: Markdown Module Split

### D177: Split markdown rendering helpers out of the core renderer

The first I19l mechanical split moved `src/extraction/markdown.rs` to a module directory:

- `src/extraction/markdown/mod.rs`: public module boundary plus core DOM-to-markdown rendering.
- `src/extraction/markdown/normalize.rs`: markdown normalization, inline spacing, escaping, URL resolution, and table-cell escaping helpers.
- `src/extraction/markdown/table.rs`: table caption, row collection, cell extraction, and markdown table line rendering.

The extraction module interface remains unchanged: callers still import `element_to_markdown`, `normalize_markdown`, and `resolve_markdown_url` through `super::markdown`.

Validation:

- `cargo fmt --check`
- `cargo test aget_extractor::tests::`
- `cargo test --test mock_site_cli aget_extractor::aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test`
- `git diff --check`

Confidence: High. This slice is mechanical, the caller-facing module interface is unchanged, focused markdown/extractor coverage passed, and the full suite passed.
