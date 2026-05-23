# D411: Browser CDP Navigation Split

Decision: split browser CDP navigation helpers into focused modules while
preserving the existing `CdpClient` navigation method names used by render,
login, state loading/export, and ignored real-Chrome smoke paths.

Boundary after the split:

- `src/browser_cdp/client/navigation/mod.rs` is the compact route for
  `navigate_and_wait` and shared navigation response/session/error helpers.
- `src/browser_cdp/client/navigation/blank.rs` owns blank-response navigation
  used when loading storage state through intercepted HTML responses.
- `src/browser_cdp/client/navigation/lifecycle.rs` owns waits for explicit
  `Page.domContentEventFired` and `Page.loadEventFired` lifecycle events.
- `src/browser_cdp/client/navigation/network_idle.rs` owns request tracking and
  the 500 ms idle window for `networkidle` waits.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
