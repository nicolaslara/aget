# D224: Browser CDP Client Test Split

Date: 2026-05-21

Decision:

- Keep `src/browser_cdp/tests/chrome.rs` as the Chrome CDP test namespace with shared mock WebSocket request/reply helpers.
- Reduce `src/browser_cdp/tests/chrome/cdp_client.rs` to a route-only module.
- Split the previous 558-line mock CDP client coverage into behavior-focused modules under `src/browser_cdp/tests/chrome/cdp_client/`.

Resulting helper modules:

- `setup.rs`: direct page connections, target discovery before attach, page-domain enablement, and auto-attach setup.
- `navigation.rs`: same-document navigation, navigation error text, and network-idle-from-load-event behavior.
- `attached_page.rs`: current-tab attached-page rendering, full-page scan ordering, and no-page-target error reporting.

Boundaries:

- This is a mechanical test-local decomposition; no browser/CDP runtime behavior changed.
- `workpads/research/tasks.md` remains the detailed executable backlog and was not compacted.

Validation:

- `cargo fmt --check`
- `cargo test browser_cdp::tests::chrome::cdp_client -- --nocapture`
- `cargo test`
- `git diff --check`
