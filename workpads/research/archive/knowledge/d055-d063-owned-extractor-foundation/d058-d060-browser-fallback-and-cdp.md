### D58: I19e starts with owned session-backed fallback extraction

Before porting the first `agent-browser` behavior, I19e inspected the local `agent-browser` snapshot paths that implement the current `aget get` fallback shape:

- `references/repos/agent-browser/cli/src/commands.rs`: parses `open`, `state load`, `get html body`, `get text body`, and `close` command shapes.
- `references/repos/agent-browser/cli/src/native/state.rs`: loads Playwright-style storage state by setting cookies and navigating to storage origins before setting local/session storage through CDP.
- `references/repos/agent-browser/cli/src/native/browser.rs`: navigates with CDP, tracks final page URL/title, and extracts DOM content through runtime evaluation.
- `references/repos/agent-browser/cli/src/native/actions.rs` and `native/element.rs`: implement `get html <selector>` as selected element `innerHTML` and `get text <selector>` as selected element text.

The current command fallback in `aget` uses this narrow sequence only after a session-backed primary extraction failure: load composed state into a temporary browser profile, open the URL, read body HTML/text, close the browser session, and clean up temp profile state. The first owned I19e slice therefore adds `OwnedBrowserAutomationBackend` with an owned fallback extraction path for static cookie-backed pages. It reuses the I19d owned fetch/HTML processing pipeline, uses structured `PlaywrightState` directly rather than an `agent-browser` state file, defaults fallback extraction to `body` to match the command fallback shape, and returns extractor metadata as `aget-owned-browser-fallback`.

This is intentionally not the full `agent-browser` replacement. `OwnedBrowserAutomationBackend` returns explicit unsupported-capability errors for Chrome/profile import and login start/finish/cancel until a real CDP/profile implementation is ported. It also does not execute page JavaScript or apply localStorage through a browser context, so localStorage-backed fallback pages still depend on future CDP/browser work.

Validation:

- `cargo test --test mock_site_cli owned_browser_fallback_replays_cookie_backed_session_without_agent_browser`

Confidence: Medium. The slice removes an `agent-browser` dependency path for static cookie-backed fallback extraction and is covered by deterministic MockSite evidence, but the high-risk browser automation work remains open.

### D59: I19d resolves owned markdown links against the page base URL

Before porting this behavior, I19d inspected Crawl4AI's link handling in `references/repos/crawl4ai/crawl4ai/markdown_generation_strategy.py`. Crawl4AI's `DefaultMarkdownGenerator` passes a base URL into its HTML-to-markdown converter and resolves relative markdown links before building citation references. Its tests in `references/repos/crawl4ai/tests/async/test_markdown_genertor.py` cover relative links and image URLs against a supplied base URL.

The owned renderer now resolves link and image URLs with the Rust `url` crate. It uses the final fetched URL as the default markdown base and honors an HTML `<base href="...">` element before extraction, matching Crawl4AI's separation between cleaned HTML and markdown generation. This keeps markdown output agent-ready when a selected content block contains relative links.

License and dependency notes:

- `url` v2.5.8 is now a direct dependency for standards-based URL joining. License: MIT OR Apache-2.0.
- No Crawl4AI source or tests were copied; the deterministic MockSite route was authored in this repo from the observed behavior target.

Validation:

- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`

Confidence: Medium-high for this slice. It closes a concrete markdown parity gap with primary-source behavior inspection and local deterministic coverage; citation/reference formatting and broader readability quality remain open.

### D60: I19e adds a minimal owned CDP renderer for localStorage-backed fallback

Before porting this behavior, I19e inspected `agent-browser`'s storage-state and CDP paths again:

- `references/repos/agent-browser/cli/src/native/state.rs` loads Playwright-style cookies first, then navigates to each storage origin and sets `localStorage`/`sessionStorage` through `Runtime.evaluate` before the target page is opened.
- `references/repos/agent-browser/cli/src/native/element.rs` extracts element HTML through CDP after resolving the selector.
- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` launches Chrome with a temporary user data directory, waits for `DevToolsActivePort`, and cleans up the temp profile after shutdown.

`OwnedBrowserAutomationBackend` now keeps the fast static fallback for cookie-only sessions but switches to a minimal owned Chrome/CDP renderer when composed state contains localStorage origins. The CDP path launches a temporary local Chrome profile, attaches to a page target, sets cookies with `Network.setCookies`, navigates each localStorage origin to set storage, opens the requested URL, optionally waits for a CSS selector through an internally generated `document.querySelector(...)` expression, then feeds the rendered document HTML back into the owned extraction/formatting pipeline. This preserves the safety rule that user-provided JavaScript waits are not executed; user input is still limited to CSS selectors and is JSON-quoted inside agent-owned CDP expressions. Owned Chrome profile temp dirs are removed on normal shutdown and are now included in orphan sweeping.

Dependency note:

- `tungstenite` v0.29.0 is now a direct dependency for the blocking local CDP WebSocket transport. License: MIT OR Apache-2.0.

Remaining I19e gaps are still substantial: Chrome/profile import, login start/finish/cancel lifecycle, current-tab attach, richer process diagnostics, screenshot/debug artifacts, and broader rendered-SPA parity. The new CDP path is intentionally scoped to fallback extraction with explicit session state and a throwaway profile.

Validation:

- `cargo test browser_cdp`
- `cargo test session::store::tests::orphan_sweep`
- `cargo test --test mock_site_cli owned_browser_fallback_replays_cookie_backed_session_without_agent_browser`
- `cargo test --test mock_site_cli owned_browser_fallback_renders_local_storage_backed_session_with_chrome -- --ignored` passed locally with system Chrome and confirmed localStorage-driven rendered DOM extraction.
- `cargo test`
- `git diff --check`

Confidence: Medium-high for this slice. The CDP command construction and cookie-only fallback path are covered by deterministic tests, and the ignored local Chrome smoke test passed on this machine. Full migration confidence still requires more lifecycle/error-path coverage before switching defaults.

