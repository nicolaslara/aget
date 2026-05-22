# D250: Mock-Site Browser Test Split

Date: 2026-05-22

Decision:

- Keep `tests/mock_site_browser.rs` as the stable integration test target route.
- Move behavior groups into focused modules under `tests/mock_site_browser/`:
  - `fallback.rs`
  - `storage.rs`
  - `rendering.rs`
  - `readiness.rs`
- Preserve test names, assertions, setup behavior, and ignored real-Chrome annotations.
- Leave `workpads/research/tasks.md` as the full planned-task backlog; this slice does not compact tasks.

Boundary:

- Runtime browser/extractor behavior is unchanged.
- This only reduces browser integration test context size for future AgetBrowser and rendered-extraction work.

Validation:

- `cargo test --test mock_site_browser`
- `cargo fmt --check`
- `cargo test`
