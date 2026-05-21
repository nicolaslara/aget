# D206: Attached Page CDP Rendering

Date: 2026-05-21

Source inspected:

- `references/repos/agent-browser/cli/src/native/browser.rs`
  - `connect_cdp_inner` and direct-page setup for page-session WebSockets.
  - `discover_and_attach_targets` for attaching existing page targets and enabling domains.
  - `enable_domains` for `Page.enable`, `Runtime.enable`, `Runtime.runIfWaitingForDebugger`, `Network.enable`, and `Target.setAutoAttach`.
  - `get_url` and `get_content` for reading `location.href` and `document.documentElement.outerHTML`.

Decision:

- Add an owned lower-level CDP renderer for already-attached/existing pages.
- The renderer connects to a supplied CDP WebSocket, attaches the preferred existing page target, enables domains, applies the existing selector/image/settle/overlay/shadow-DOM capture behavior, and returns final URL plus HTML.
- It must not launch Chrome, create a target, navigate, close the page, or close the browser. That keeps it suitable for future current-tab work where the browser belongs to the user or another explicit owner.

Boundaries:

- This is a current-tab prerequisite, not a public CLI/API surface.
- If no page target exists, the renderer returns a stable extraction failure instead of creating `about:blank`; public current-tab UX should decide how to report or remediate that later.
- Endpoint discovery and user consent UX remain follow-up work.

Validation:

- `cargo fmt --check`
- `cargo test browser_cdp::tests::chrome::`
- `cargo test`
- `git diff --check`
