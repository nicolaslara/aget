# D292: Aget Facade Session Orchestration Split

Date: 2026-05-22

## Decision

Split session/import/login/authorization orchestration out of `src/aget/mod.rs` into `src/aget/sessions.rs` without changing the public `AgetWith` API.

## Boundary

- `src/aget/mod.rs` remains the facade wiring entrypoint. It owns module routing, public re-exports, default `Aget` type alias, constructors, backend replacement helpers, timeout/home access, get-request creation, and shared facade helpers.
- `src/aget/sessions.rs` owns `AgetWith` methods for session list/load/delete, cmux and Chrome import, authorization verification, session composition, and login start/finish/cancel orchestration.
- `src/aget/authorize.rs`, `src/aget/backends.rs`, `src/aget/current_tab.rs`, `src/aget/get_request.rs`, and `src/aget/session_store.rs` keep their existing roles.

## Safety And Compatibility Notes

- This is a mechanical facade decomposition only. Default owned backend wiring, compatibility backend opt-ins, session persistence, import behavior, authorization warnings, compose validation, and login cleanup behavior are unchanged.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test --test aget_api`
- `cargo test --test session_cli`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
