# D418: Session CLI Command Definition Split

Decision: split `aget session` CLI command definitions into focused modules
while preserving all public command type names re-exported from `src/cli.rs`
and `src/lib.rs`.

Boundary after the split:

- `src/cli/session/mod.rs` routes session subcommands and re-exports command
  types.
- `src/cli/session/browser.rs` owns `BrowserChoice` and its string mapping.
- `src/cli/session/authorize.rs` owns `session authorize` arguments.
- `src/cli/session/login.rs` owns `session login` start/finish/cancel
  arguments.
- `src/cli/session/import.rs` owns `session import` cmux/browser/chrome
  arguments.
- `src/cli/session/basic.rs` owns inspect, delete, and compose arguments.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
