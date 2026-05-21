# D199: Direct page CDP session support

## Decision

`AgetBrowser` CDP plumbing now distinguishes browser-level WebSocket connections from direct page WebSocket connections.

Primary source:

- `references/repos/agent-browser/cli/src/native/browser.rs`
- `BrowserManager::connect_cdp_direct`
- `BrowserManager::connect_cdp_inner`
- `BrowserManager::enable_domains_direct`
- `BrowserManager::discover_and_attach_targets`

`agent-browser` treats direct page CDP connections as already attached to one page and enables Page, Runtime, and Network domains without a flattened `sessionId`. Browser-level connections still discover targets, attach with `Target.attachToTarget`, and use the returned `sessionId`.

## Boundaries

- Direct page detection is based on Chrome DevTools page/webview WebSocket paths.
- Direct page sessions use empty internal target/session IDs and route CDP commands without `sessionId`.
- Browser-level target creation, attach, close, and rendering behavior remains unchanged.
- `Browser.close` is a no-op for direct page connections because page targets do not expose browser-level close semantics.
- This is a lower-level current-tab prerequisite; it does not add a public `aget current-tab` CLI surface.

## Validation

- Added a deterministic mock CDP test proving direct page domain enabling omits `sessionId`.
- `cargo fmt --check`
- `cargo test browser_cdp::tests`
- `cargo test`
- `git diff --check`
