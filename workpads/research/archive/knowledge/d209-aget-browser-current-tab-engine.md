# D209: AgetBrowser Current Tab Engine

Date: 2026-05-21

Source inspected:

- `references/repos/agent-browser/cli/src/native/browser.rs`: external CDP connection setup, current page target discovery/attachment, connection liveness, and external-browser close behavior.
- Reuses D208 source inspection for CDP URL discovery order.
- Reuses D206/D207 source inspection for attached-page content capture.

Decision:

- Add a crate-internal `AgetBrowser` current-tab render seam that composes explicit local CDP-port discovery with attached-page rendering.
- Preserve public CLI/API behavior while current-tab consent and command UX remain undecided.
- Require callers to provide the local debugging port explicitly; do not add ambient port/profile scanning at this seam.
- Return the resolved CDP WebSocket URL with the rendered page result for internal provenance/debugging.

Boundaries:

- This is not a public current-tab feature yet.
- The composed path does not navigate, create pages, close pages, or close the browser.
- The existing lower-level attached-page renderer still owns target attachment, waits, overlay cleanup, optional shadow-DOM flattening, final URL capture, and HTML capture.

Validation:

- `cargo fmt --check`
- `cargo test aget_browser::tests::`
- `cargo test browser_cdp::tests::chrome::cdp_client::browser_cdp_render_attached_page`
- `cargo test browser_cdp::tests::discovery::`
- `cargo test`
- `git diff --check`
