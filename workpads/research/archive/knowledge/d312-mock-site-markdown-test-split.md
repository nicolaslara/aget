# D312: Mock-Site Markdown Test Split

Date: 2026-05-22

## Decision

Split `tests/mock_site_cli/aget_extractor/markdown.rs` into smaller behavior-owned child modules while preserving the existing parent integration-test route.

## Boundary

- `tests/mock_site_cli/aget_extractor/markdown.rs` now only routes the markdown assertion groups and owns the shared `markdown_content` helper.
- `tests/mock_site_cli/aget_extractor/markdown/tables_base.rs` covers core markdown table rendering, `ignore_tables`, `bypass_tables`, and base-link resolution.
- `tests/mock_site_cli/aget_extractor/markdown/inline_blocks.rs` covers inline/block markdown constructs, emphasis/quote marker options, sup/sub, `ignore_emphasis`, and `escape_snob`.
- `tests/mock_site_cli/aget_extractor/markdown/lists_code.rs` covers nested lists, `ul_item_mark`, code whitespace, and ordered-list start behavior.
- `tests/mock_site_cli/aget_extractor/markdown/links_images.rs` covers link, image, mailto, internal-link, automatic-link, image-alt, raw-image, sized-image, default-alt, and protected-link behavior.

## Safety And Compatibility Notes

- This is a mechanical test decomposition only; extraction runtime behavior is unchanged.
- The existing parent mock-site integration test continues to call `markdown::assert_markdown_rendering`.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.
- The split reduces the markdown test route from 511 lines to a 38-line parent plus smaller behavior modules; the largest child is currently 230 lines.

## Validation

- `cargo fmt`
- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
