# D216: AgetBrowser Current-Tab Split

Date: 2026-05-21

Decision:

- Keep `src/aget_browser.rs` as the small public engine wrapper for owned browser behavior.
- Move current-tab/CDP endpoint request and result types plus endpoint discovery, attached-page rendering, and composed current-tab rendering methods to `src/aget_browser/current_tab.rs`.
- Move internal `AgetBrowser` engine tests to `src/aget_browser/tests.rs` without weakening assertions.

Boundary:

- This is a mechanical decomposition only. It does not change `AgetBrowser`, `AgetBrowserBackend`, or current-tab behavior.
- The root `aget_browser` module re-exports only the crate-visible `CurrentTabRequest` type needed by the backend adapter; lower-level current-tab test-only helpers stay in the child module boundary.
- `workpads/research/tasks.md` remains the un-compacted source of truth for planned and completed tasks.

Validation:

- `cargo fmt --check`
- `cargo test aget_browser::tests`
- `cargo test`
- `git diff --check`
