# D388: Aget API Support Split

Decision: split the Aget API integration-test support helper into
behavior-focused modules without changing exported helper names or test
behavior.

Boundary after the split:

- `tests/aget_api/support.rs` keeps the compact router and public test re-exports.
- `tests/aget_api/support/session.rs` owns cookie-session and pending-login
  fixture builders.
- `tests/aget_api/support/session_store.rs` owns the in-memory session-store
  backend.
- `tests/aget_api/support/extractor_backend.rs` owns extractor fakes for static,
  failing, and authorization verification paths.
- `tests/aget_api/support/browser_backend.rs` owns the browser automation and
  fallback fake backend.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
