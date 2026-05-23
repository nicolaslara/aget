# D406: Browser CDP Client Transport Split

Decision: split owned CDP client transport helpers into connection setup and
message/command handling modules while keeping the existing
`browser_cdp::client::transport::{...}` route for callers and tests.

Boundary after the split:

- `transport/mod.rs` owns the route, WebSocket config, timeout remaining helper,
  CDP I/O error mapping, and direct page WebSocket URL classification.
- `transport/connection.rs` owns CDP WebSocket connection setup and socket
  timeout configuration.
- `transport/message.rs` owns command serialization, response matching, frame
  parsing, automatic alert/beforeunload handling, keepalive pings, and test
  keepalive tuning.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
