# D320: Login-Finish Session CLI Test Split

Date: 2026-05-22

## Decision

Split `tests/session_cli/login/finish.rs` into smaller scenario-owned child modules while preserving the existing login-finish integration-test route.

## Boundary

- `tests/session_cli/login/finish.rs` now only routes login-finish scenario modules.
- `tests/session_cli/login/finish/success.rs` covers scoped state saving, artifact cleanup, and raw state cleanup after a successful finish.
- `tests/session_cli/login/finish/merge.rs` covers replacing the current login scope while preserving unrelated existing session scope.
- `tests/session_cli/login/finish/failures.rs` covers close failure preservation and provider-only state rejection.

## Safety And Compatibility Notes

- This is a mechanical test decomposition only; session login CLI behavior is unchanged.
- The parent route remains under the existing `login::finish` integration-test module and uses explicit `#[path = "..."]` child routes.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.
- The parent test route dropped from 302 lines to 6 lines; all child modules are currently 116 lines or less.

## Validation

- `cargo fmt`
- `cargo test --test session_cli login::finish`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
