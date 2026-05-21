# Knowledge Archive D179: CDP Client Module Split

### D179: Split CDP client transport, navigation, and state helpers

The first I19n mechanical split moved `src/browser_cdp/client.rs` to a module directory:

- `src/browser_cdp/client/mod.rs`: `CdpClient`, `PageSession`, target creation/attachment, domain enabling, shadow-root setup, and close-browser wrapper.
- `src/browser_cdp/client/transport.rs`: WebSocket connection, command serialization, response matching, CDP message reads, socket timeouts, and CDP I/O error mapping.
- `src/browser_cdp/client/navigation.rs`: page navigation, blank-response navigation for storage export, network-idle waits, selector/image waits, overlay cleanup, and string evaluation.
- `src/browser_cdp/client/state.rs`: Playwright state load/export, cookie collection, and localStorage/sessionStorage export.

The effective `browser_cdp`-module visibility of client methods is preserved with `pub(in crate::browser_cdp)` where those methods are used by sibling CDP modules and tests.

Validation:

- `cargo fmt --check`
- `cargo test browser_cdp::tests::`
- `cargo test`
- `git diff --check`

Confidence: High. This is a mechanical split, focused CDP coverage passed, and the full suite passed.
