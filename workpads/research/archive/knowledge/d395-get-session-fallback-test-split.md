# D395: Get Session Fallback Test Split

Decision: split get CLI session fallback tests into behavior-focused modules
without changing fallback coverage or assertion behavior.

Boundary after the split:

- `tests/get_cli/session.rs` still routes through the `fallback` module.
- `session/fallback/mod.rs` is the compact module route.
- `fallback/redaction.rs` owns session-state secret redaction across errors,
  metadata, and backend artifacts.
- `fallback/success.rs` owns successful agent-browser fallback with composed
  state, metadata, warnings, log, and temp cleanup assertions.
- `fallback/no_fallback.rs` owns the unauthenticated backend-failure path that
  must not invoke agent-browser fallback.
- `fallback/close_failure.rs` owns preservation of the original sanitized
  Crawl4AI error when fallback close fails.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
