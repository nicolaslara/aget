# D405: Cmux Import Test Split

Decision: split cmux session import tests into deterministic CLI coverage and
ignored real-smoke recipes while keeping the existing
`session_cli::imports_cmux` route.

Boundary after the split:

- `tests/session_cli/imports_cmux.rs` owns only the test route.
- `tests/session_cli/imports_cmux/deterministic.rs` owns mocked cmux import and
  missing-backend assertions.
- `tests/session_cli/imports_cmux/real_smoke.rs` owns the ignored live cmux
  loopback smoke recipes.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
