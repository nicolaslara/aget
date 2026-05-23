# D394: Get Session Replay Test Split

Decision: split get CLI session replay tests into behavior-focused modules
without changing replay coverage or assertion behavior.

Boundary after the split:

- `tests/get_cli/session.rs` still routes through the `replay` module.
- `session/replay/mod.rs` is the compact module route.
- `replay/single.rs` owns named-session state replay and sensitive output
  artifact metadata assertions.
- `replay/scope.rs` owns replay-scope privacy rejection coverage.
- `replay/repeated.rs` owns repeated-session composition for command and
  top-level URL alias paths.
- `replay/provider_flow.rs` owns the local app/provider cookie flow.
- `replay/sensitivity.rs` owns the rule that session-backed output is sensitive
  even if stored session metadata says otherwise.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
