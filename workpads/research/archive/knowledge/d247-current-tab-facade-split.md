# D247: Current-Tab Facade Split

Date: 2026-05-22

Decision:

- Move `CurrentTabOptions` and `AgetWith::current_tab` orchestration from `src/aget/mod.rs` into `src/aget/current_tab.rs`.
- Keep `aget::CurrentTabOptions` re-exported from the facade module so callers keep the same public path.
- Keep current-tab consent, warning, extraction, and backend request behavior unchanged.
- Leave `workpads/research/tasks.md` as the full planned-task backlog; this slice does not compact tasks.

Boundary:

- `src/aget/mod.rs` remains the public facade and session/get orchestration surface.
- `src/aget/current_tab.rs` owns current-tab DTO defaults, private-content consent enforcement, CDP render request composition, and direct extraction finalization.

Validation:

- `cargo test current_tab`
- `cargo fmt --check`
- `cargo test`
