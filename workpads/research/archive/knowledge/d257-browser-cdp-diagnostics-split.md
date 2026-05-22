# D257: Browser CDP Diagnostics Split

## Decision

Split Chrome startup diagnostics helpers out of `src/browser_cdp/discovery.rs`.

## Boundary

- `src/browser_cdp/discovery.rs` keeps DevToolsActivePort polling, existing-profile attach, `/json/version`, `/json/list`, direct `/devtools/browser` discovery, WebSocket host rewriting, and profile-browser shutdown polling.
- `src/browser_cdp/discovery/diagnostics.rs` owns Chrome stderr DevTools URL fallback parsing, startup error classification, requires-user-action matching, relevant stderr filtering, sandbox hints, generic stderr tails, and no-stderr startup hints.

## Compatibility

The split is mechanical. Chrome process callers and browser CDP discovery tests continue to access diagnostics through the `discovery` module, and existing startup error messages, hint text, and fallback URL parsing are unchanged.

## Validation

- `cargo test browser_cdp::tests::discovery`

Standard validation for the stable point is recorded in the task status/commit that includes this decision.
