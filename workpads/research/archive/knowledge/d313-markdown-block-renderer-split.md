# D313: Markdown Block Renderer Split

Date: 2026-05-22

## Decision

Split block-level markdown rendering helpers out of `src/extraction/markdown/mod.rs` into `src/extraction/markdown/block.rs`.

## Boundary

- `src/extraction/markdown/mod.rs` remains the caller-facing markdown renderer route and keeps `element_to_markdown`, node dispatch, inline-tag decisions, and the small `only_text` eligibility table.
- `src/extraction/markdown/block.rs` owns heading, paragraph/block, horizontal-rule, list, definition-list, code-block, blockquote, and structural-block helpers.
- Existing inline rendering, table rendering, normalization, and writer state modules remain unchanged.

## Safety And Compatibility Notes

- This is a mechanical production-code decomposition only; owned extraction runtime behavior is unchanged.
- The public extraction module interface and `element_to_markdown` call sites are unchanged.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.
- The markdown route dropped from 345 lines to 188 lines; the new block helper is 168 lines.

## Validation

- `cargo fmt`
- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
