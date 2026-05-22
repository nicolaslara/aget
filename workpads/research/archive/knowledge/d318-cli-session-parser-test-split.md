# D318: CLI Session Parser Test Split

Date: 2026-05-22

## Decision

Split `src/cli/tests/session.rs` into smaller command-surface child modules while preserving the existing CLI parser test route.

## Boundary

- `src/cli/tests/session.rs` now only routes session parser test modules.
- `src/cli/tests/session/inspect_compose.rs` covers session inspect and compose parser assertions.
- `src/cli/tests/session/authorize.rs` covers session authorize parser assertions, including Chrome profile aliasing.
- `src/cli/tests/session/import.rs` covers cmux, browser, and Chrome import parser assertions.
- `src/cli/tests/session/login.rs` covers login start, finish, and cancel parser assertions.

## Safety And Compatibility Notes

- This is a mechanical test decomposition only; CLI parser behavior and public command names are unchanged.
- The parent test route remains under the existing `cli::tests::session` module.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.
- The parent test route dropped from 309 lines to 4 lines; all child modules are currently 137 lines or less.

## Validation

- `cargo fmt`
- `cargo test cli::tests::session --lib`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
