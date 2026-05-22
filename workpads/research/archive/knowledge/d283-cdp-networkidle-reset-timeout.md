# D283: CDP Networkidle Reset and Timeout Coverage

Decision: owned CDP `PageWaitUntil::NetworkIdle` behavior is now covered for the remaining source-backed `agent-browser` networkidle edge cases.

Source inspection:

- `references/repos/agent-browser/cli/src/native/browser.rs`: `poll_network_idle`.
- `agent-browser` starts a 500 ms quiet window only after readiness/no pending requests, resets that window when a new `Network.requestWillBeSent` arrives, starts a new quiet window when pending requests empty, and returns `Timeout waiting for networkidle` when the overall timeout expires first.

Implementation boundary:

- Production owned CDP navigation code already matched this bounded behavior: it tracks same-session `Network.requestWillBeSent`, `Network.loadingFinished`, and `Network.loadingFailed`, resets `idle_since` on new requests, restarts it when in-flight requests empty, and uses the caller timeout as the overall deadline.
- This slice adds deterministic mock-CDP tests rather than changing runtime behavior.
- This remains bounded network-idle readiness; it does not claim full Playwright parity for long-polling, websockets, service workers, virtual scrolling, or app-specific readiness.

Validation:

- `cargo test browser_cdp_networkidle`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
