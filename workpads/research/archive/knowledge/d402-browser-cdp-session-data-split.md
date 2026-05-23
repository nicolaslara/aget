# D402: Browser CDP Session-Data Split

Decision: split the owned CDP session-data helpers into behavior-focused
modules without changing the helper names re-exported through
`browser_cdp::client`.

Boundary after the split:

- `session_data/mod.rs` owns the module route and re-exports the helper surface.
- `session_data/cookies.rs` owns Playwright-to-CDP cookie payloads, CDP-to-
  Playwright cookie parsing, and cookie dedupe.
- `session_data/storage.rs` owns allowed storage origin expansion, frame-tree
  origin collection, origin allowlist matching, and Runtime.evaluate storage
  parsing.
- `session_data/targets.rs` owns preferred page/webview target selection and
  internal Chrome target filtering.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
