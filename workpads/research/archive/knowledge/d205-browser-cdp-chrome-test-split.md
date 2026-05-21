# D205: Browser CDP Chrome Test Split

Date: 2026-05-21

Decision:

- Keep `src/browser_cdp/tests/chrome.rs` as the Chrome test namespace and shared mock WebSocket helper location.
- Split the test implementation into behavior-focused helper modules under `src/browser_cdp/tests/chrome/`.
- Preserve all existing test behavior and names; this is a mechanical test-local decomposition, not a browser/CDP behavior change.

Resulting helper modules:

- `cdp_client.rs`: direct page CDP sessions, target discovery/auto-attach, navigation result handling, and network-idle load-event coverage.
- `chrome_process.rs`: Chrome launch retry coverage.
- `real_chrome.rs`: ignored real-Chrome profile import and headed login state export smokes.

Boundaries:

- Mock WebSocket request/reply helpers stay private to the Chrome test namespace.
- `workpads/research/tasks.md` remains the detailed executable backlog and was not compacted.

Validation:

- `cargo fmt --check`
- `cargo test browser_cdp::tests::chrome::`
- `cargo test`
- `git diff --check`
