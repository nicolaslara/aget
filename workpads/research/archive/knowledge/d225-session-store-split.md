# D225: Session Store Split

Date: 2026-05-21

Decision:

- Keep `src/session/store/mod.rs` as the public `session::store` API surface for `SessionStore`.
- Move private file and directory permission helpers into `src/session/store/permissions.rs`.
- Move orphaned temp-file/profile sweeping into `src/session/store/cleanup.rs`.
- Move the existing unit tests into `src/session/store/tests.rs`.

Boundaries:

- This is a mechanical production-code decomposition; session layout, save/load/list/delete, private-permission, and orphan-sweep behavior did not intentionally change.
- `SessionStore` remains exported through `aget::session::SessionStore`.
- `workpads/research/tasks.md` remains the detailed executable backlog and was not compacted.

Validation:

- `cargo fmt --check`
- `cargo test session::store -- --nocapture`
- `cargo test`
- `git diff --check`
