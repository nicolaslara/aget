# D391: Browser Render Capture Split

Decision: split the browser/CDP render module into a compact orchestration
route and a focused attached-page capture helper without changing public
module exports.

Boundary after the split:

- `src/browser_cdp/render/mod.rs` owns render request/result types,
  `PageWaitUntil`, browser launch/attach orchestration, page-domain setup,
  context overrides, state loading, navigation, and browser shutdown.
- `src/browser_cdp/render/capture.rs` owns attached-page readiness and capture:
  full-page scan, selector wait, image readiness, settle delay, iframe
  processing, overlay cleanup, final URL evaluation, shadow-DOM flattening
  fallback, outerHTML fallback, and warning collection.

The `browser_cdp::render_page`, `browser_cdp::render_attached_page`, and
request/result type paths remain stable through the existing
`src/browser_cdp.rs` re-exports.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
