# D195: Browser CDP Test Split

The browser CDP unit tests were mechanically split by behavior so future `AgetBrowser`/CDP work can load the relevant coverage without loading all state conversion, Chrome lifecycle, script, and discovery tests together.

Resulting boundaries:

- `src/browser_cdp/tests.rs`: test module router plus shared environment-variable and cleanup helpers.
- `src/browser_cdp/tests/state.rs`: CDP cookie payload, Playwright state conversion, candidate origins, and runtime storage parsing coverage.
- `src/browser_cdp/tests/chrome.rs`: Chrome launch retry plus ignored real Chrome profile import and headed login export smoke coverage.
- `src/browser_cdp/tests/scripts.rs`: local/session storage expressions, page target selection, selector waits, overlay cleanup, and shadow DOM scripts.
- `src/browser_cdp/tests/discovery.rs`: DevToolsActivePort, stderr fallback, startup diagnostics, and `/json/*` discovery coverage.

The split did not intentionally change CDP behavior, browser lifecycle behavior, ignored smoke requirements, or test assertions.

Validation:

- `cargo fmt --check`
- `cargo test browser_cdp::tests`
- `cargo test`
- `git diff --check`
