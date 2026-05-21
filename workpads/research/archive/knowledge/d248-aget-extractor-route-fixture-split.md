# D248: AgetExtractor Route Fixture Split

Date: 2026-05-22

Decision:

- Keep `tests/mock_site_cli/aget_extractor_site.rs` as the stable `aget_extractor_parity_site()` route composer.
- Move dense route HTML into behavior-owned modules under `tests/mock_site_cli/aget_extractor_site/`:
  - `formats_options.rs`
  - `main_content.rs`
  - `selectors.rs`
  - `markdown.rs`
  - `cleanup.rs`
- Preserve the AgetExtractor parity test entrypoint and assertions unchanged.
- Leave `workpads/research/tasks.md` as the full planned-task backlog; this slice does not compact tasks.

Boundary:

- Runtime behavior is unchanged.
- This only reduces fixture context size for future AgetExtractor parity work.

Validation:

- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice`
- Route/asset string comparison between `HEAD:tests/mock_site_cli/aget_extractor_site.rs` and the split fixture matched.
- `cargo fmt --check`
- `cargo test`
