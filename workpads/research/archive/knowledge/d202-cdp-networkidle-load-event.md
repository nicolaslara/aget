# D202: CDP network-idle load-event readiness

## Decision

`AgetBrowser` CDP network-idle waiting now accepts `Page.loadEventFired` as a readiness event when no network requests are in flight.

Primary source:

- `references/repos/agent-browser/cli/src/native/browser.rs`
- `BrowserManager::wait_for_network_idle`
- `poll_network_idle`

`agent-browser` tracks pending network requests and starts the 500 ms quiet window when `Page.loadEventFired` arrives with no pending requests. The owned CDP client already handled `Page.domContentEventFired`; this slice preserves that behavior and adds the load-event path.

## Boundaries

- `Page.domContentEventFired` still starts the network-idle quiet window when no requests are in flight.
- `Page.loadEventFired` now also starts that quiet window when no requests are in flight.
- Request tracking for `Network.requestWillBeSent`, `Network.loadingFinished`, and `Network.loadingFailed` remains unchanged.
- This is rendered-page readiness parity for the existing owned browser path and does not add a public current-tab CLI surface.

## Validation

- Added deterministic mock CDP coverage for `networkidle` completion after a load-only readiness event.
- `cargo fmt --check`
- `cargo test browser_cdp::tests`
- `cargo test`
- `git diff --check`
