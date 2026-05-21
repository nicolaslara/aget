# D211: Current-Tab Compatibility Env Boundary

Date: 2026-05-21

Source inspected:

- `references/repos/agent-browser/cli/src/native/browser.rs`: external CDP connection setup, current-page target discovery/attachment, and external-browser close behavior.
- Reuses D210 source inspection for the public current-tab consent boundary.

Decision:

- Keep `Aget::current_tab` and `aget current-tab` on the owned `AgetBrowserBackend` even when `AGET_AGENT_BROWSER_COMMAND` selects command-backed compatibility adapters for login/import/fallback behavior.
- Preserve the explicit command adapters for their existing compatibility surfaces.
- Treat current-tab as an owned-only public path because the consent-gated command is implemented through local CDP endpoint discovery and attached-page rendering, not through the legacy agent-browser command contract.

Boundaries:

- This does not remove the `agent-browser` command adapter for login/import/fallback compatibility.
- This does not add ambient browser or port discovery.
- This does not change current-tab's explicit `--cdp-port` and `--allow-private-content` requirements.

Validation:

- `cargo fmt --check`
- `cargo test --test cli current_tab`
- `cargo test`
- `git diff --check`
