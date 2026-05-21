# D249: Session Authorize Test Split

Date: 2026-05-22

Decision:

- Keep `tests/session_cli/authorize.rs` as the stable parent route from `tests/session_cli.rs`.
- Move individual authorization scenarios into behavior-owned modules under `tests/session_cli/authorize/`:
  - `success.rs`
  - `verification_failed.rs`
  - `requires_user_action.rs`
  - `unsupported_browser.rs`
  - `reimport.rs`
- Preserve command arguments, mocked backend behavior, JSON assertions, session-store assertions, and cookie echo assertions.
- Leave `workpads/research/tasks.md` as the full planned-task backlog; this slice does not compact tasks.

Boundary:

- Runtime CLI behavior is unchanged.
- This only reduces session authorization test context size for future auth/session work.

Validation:

- `cargo test session_authorize`
- `cargo fmt --check`
- `cargo test`
