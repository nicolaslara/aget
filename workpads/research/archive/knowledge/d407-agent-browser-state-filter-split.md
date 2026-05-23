# D407: Agent-Browser State Filter Split

Decision: split agent-browser state filtering into model, domain matching, and
session-filtering modules while keeping the existing
`session::agent_browser::state_filter::{...}` route for import code and tests.

Boundary after the split:

- `state_filter/mod.rs` owns the route and re-export surface.
- `state_filter/model.rs` owns raw agent-browser state, cookie, origin, and
  expiration deserialization shapes.
- `state_filter/domains.rs` owns domain normalization, allowlist matching, and
  origin-host extraction.
- `state_filter/filter.rs` owns raw-state file loading, cookie/storage
  allowlist filtering, duplicate-conflict detection, session construction, and
  the Playwright-state adapter.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
