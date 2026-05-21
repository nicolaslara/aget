# D207: AgetBrowser Attached Page Seam

Date: 2026-05-21

Source inspected:

- Reuses D206 source inspection of `references/repos/agent-browser/cli/src/native/browser.rs` for existing-page attach, domain enablement, `get_url`, and `get_content`.

Decision:

- Expose the attached-page CDP renderer through a crate-internal `AgetBrowser` engine method.
- Keep the public CLI/API unchanged until the consent prompt, endpoint discovery, and current-tab command UX are chosen.
- Require callers at this seam to pass an explicit CDP WebSocket URL, so endpoint ownership is not implicit or ambient.

Boundaries:

- This is not a public current-tab feature yet.
- The seam is direct-engine coverage for future orchestration work; it does not route through the `Aget` facade or command compatibility backends.
- The lower-level renderer still owns the no-launch/no-navigation/no-close behavior.

Validation:

- `cargo fmt --check`
- `cargo test aget_browser::tests::`
- `cargo test browser_cdp::tests::chrome::cdp_client::browser_cdp_render_attached_page`
- `cargo test`
- `git diff --check`
