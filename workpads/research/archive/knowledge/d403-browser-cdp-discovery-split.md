# D403: Browser CDP Discovery Split

Decision: split owned CDP discovery into startup active-port, endpoint
discovery, and existing-profile attach/shutdown modules without changing the
`browser_cdp::discovery::{...}` helper paths used by callers and tests.

Boundary after the split:

- `discovery/mod.rs` owns the route and re-export surface.
- `discovery/active_port.rs` owns `DevToolsActivePort` parsing and startup wait
  logic, including Chrome stderr fallback URLs.
- `discovery/endpoint.rs` owns `/json/version`, `/json/list`, direct WebSocket
  fallback discovery, WebSocket URL host rewriting, and discovery errors.
- `discovery/profile.rs` owns existing-profile attach retries, stale
  `DevToolsActivePort` cleanup, and profile-browser shutdown polling.
- `discovery/diagnostics.rs` continues to own Chrome startup error
  classification.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
