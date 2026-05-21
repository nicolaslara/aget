# D201: CDP same-document navigation handling

## Decision

`AgetBrowser` CDP navigation now mirrors the `agent-browser` `Page.navigate` response boundary for same-document navigations and navigation result failures.

Primary source:

- `references/repos/agent-browser/cli/src/native/browser.rs`
- `BrowserManager::navigate`

`agent-browser` checks the typed `Page.navigate` result before waiting for lifecycle events. If Chrome returns no `loader_id`, the navigation was same-document, such as hash routing, and page lifecycle events such as `Page.loadEventFired`, `Page.domContentEventFired`, or network-idle completion are not expected. It also surfaces `error_text` from the navigate result as a navigation failure.

## Boundaries

- `AgetBrowser` still waits for the configured lifecycle event on full-document navigations with a `loaderId`.
- Same-document navigation responses return immediately for load, domcontentloaded, and networkidle waits.
- CDP command-level errors are still reported as before.
- `Page.navigate` result-level `errorText` now becomes a stable extraction failure instead of being ignored while waiting for events.
- This is lower-level rendered-page readiness parity and does not add a public current-tab CLI surface.

## Validation

- Added deterministic mock CDP coverage proving networkidle does not wait after a no-`loaderId` navigation result.
- Added deterministic mock CDP coverage proving navigate `errorText` is reported.
- `cargo fmt --check`
- `cargo test browser_cdp::tests`
- `cargo test`
- `git diff --check`
