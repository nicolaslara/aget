# D399: Browser CDP Client Page Split

Decision: split the owned CDP client route so page target/session lifecycle and
Runtime.evaluate exception handling live in smaller behavior-focused modules
without changing browser render, state export, login, or CDP test call paths.

Boundary after the split:

- `src/browser_cdp/client/mod.rs` owns the `CdpClient` state type and routes
  client submodules.
- `src/browser_cdp/client/page.rs` owns page target creation/attachment,
  domain enabling, shadow-root bootstrap, request identity overrides, and page
  or browser shutdown helpers.
- `src/browser_cdp/client/runtime.rs` owns Runtime.evaluate exception
  classification shared by navigation readiness, capture, and storage helpers.

The `client::{CdpClient, PageSession}` path and existing browser-CDP method
callers are preserved.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
