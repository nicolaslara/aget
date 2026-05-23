# D400: Browser CDP Transport Test Split

Decision: split the owned CDP transport tests into behavior-focused modules
without changing the assertions or the `cdp_client::transport` test route.

Boundary after the split:

- `transport/mod.rs` owns the route and tiny shared WebSocket URL helper.
- `transport/config.rs` owns CDP WebSocket configuration coverage.
- `transport/keepalive.rs` owns keepalive ping behavior while waiting for
  command responses.
- `transport/dialogs.rs` owns automatic alert/beforeunload handling and the
  explicit prompt boundary.
- `transport/frames.rs` owns binary, invalid-binary, and malformed-frame
  parsing behavior.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
