# D200: CDP target discovery and auto-attach parity

## Decision

`AgetBrowser` CDP attach setup now mirrors the `agent-browser` browser-level target setup used for existing pages.

Primary source:

- `references/repos/agent-browser/cli/src/native/browser.rs`
- `BrowserManager::discover_and_attach_targets`
- `BrowserManager::enable_domains`
- `BrowserManager::enable_domains_direct`

`agent-browser` enables target discovery before reading `Target.getTargets`, then attaches to page/webview targets with flattened sessions. After Page, Runtime, and Network domains are enabled for a non-direct page session, it best-effort enables `Target.setAutoAttach` with `flatten: true` so subtargets such as cross-origin frames can be discovered without failing engines that do not support the call.

## Boundaries

- Browser-level existing-page attach now sends `Target.setDiscoverTargets { discover: true }` before `Target.getTargets`.
- Non-direct page sessions best-effort send `Target.setAutoAttach` with `autoAttach: true`, `waitForDebuggerOnStart: false`, and `flatten: true` after domain setup.
- Direct page/webview WebSocket connections keep the D199 behavior: no flattened `sessionId`, no browser-level target discovery, and no `Target.setAutoAttach`.
- This is lower-level CDP readiness parity and does not add a public current-tab CLI surface.

## Validation

- Added deterministic mock CDP coverage for discovery-before-attach command order.
- Added deterministic mock CDP coverage for non-direct `Target.setAutoAttach` session scoping.
- `cargo fmt --check`
- `cargo test browser_cdp::tests`
- `cargo test`
- `git diff --check`
