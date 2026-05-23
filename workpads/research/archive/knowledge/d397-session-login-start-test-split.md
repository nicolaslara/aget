# D397: Session Login Start Test Split

Decision: split `aget session login start` CLI tests into behavior-focused
modules without changing coverage or assertion behavior.

Boundary after the split:

- `tests/session_cli/login.rs` still routes through the `start` module.
- `login/start/mod.rs` is the compact module route.
- `start/success.rs` owns the started-login happy path, pending metadata,
  warning, allowed domains, next command, profile path, and agent-browser log
  assertions.
- `start/validation.rs` owns HTTPS-only URL validation before browser startup.
- `start/scope.rs` owns exact non-`www` host scope behavior.
- `start/conflict.rs` owns injected-session conflict rejection before browser
  startup.
- `start/duplicate.rs` owns duplicate pending-flow rejection without
  overwriting or closing the original flow.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
