# D208: AgetBrowser CDP Endpoint Discovery

Date: 2026-05-21

Source inspected:

- `references/repos/agent-browser/cli/src/native/browser.rs`: `connect_cdp_inner`, `connect_auto`, and external-CDP close behavior.
- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`: `auto_connect_cdp` and active-port resolution order.
- `references/repos/agent-browser/cli/src/native/cdp/discovery.rs`: CDP discovery order and WebSocket URL rewriting.

Decision:

- Expose owned CDP endpoint discovery through a crate-internal `AgetBrowser` engine method.
- Preserve public CLI/API behavior while current-tab consent and command UX remain undecided.
- Require callers at this seam to provide the local CDP debugging port explicitly.
- Reuse the owned discovery order: `/json/version`, `/json/list`, then direct `/devtools/browser` WebSocket verification.

Boundaries:

- This is not a public current-tab feature yet.
- The seam does not auto-scan common ports or profile directories; ambient auto-connect belongs behind a future consented command/API decision.
- The attached-page renderer still owns page attachment and HTML capture after a WebSocket URL is known.

Validation:

- `cargo fmt --check`
- `cargo test aget_browser::tests::`
- `cargo test browser_cdp::tests::discovery::`
- `cargo test`
- `git diff --check`
