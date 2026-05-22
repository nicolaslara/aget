# D116-D120: Chrome And Overlay Cleanup

### D116: I19e retries owned Chrome launch startup failures

The next browser lifecycle slice ports `agent-browser`'s Chrome launch retry behavior. Before changing the owned browser backend, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` at local commit `3bb1d43`, where `launch_chrome` retries `try_launch_chrome` up to three times and waits 500ms between failed attempts before returning the last startup error.

`OwnedBrowserAutomationBackend` now applies the same three-attempt, 500ms retry policy when launching local Chrome for fallback rendering, Chrome profile import, and dedicated login browsers. Each attempt still removes stale `DevToolsActivePort`, captures fresh stderr, uses the existing process-group cleanup path on failure, and preserves the final startup classification/sandbox hints when all attempts fail.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test chrome_launch_retries_after_early_startup_exit`
- `cargo test`

Confidence: High for transient early-exit retry behavior. The behavior is source-backed and covered by a deterministic fake-Chrome test that fails the first launch, writes `DevToolsActivePort` on the second launch, and proves the owned backend returns the discovered CDP URL. Broader real-Chrome/keychain smoke coverage remains separate I19e work.

### D117: I19e removes stale DevToolsActivePort files after failed attach

The next existing-profile attach slice ports `agent-browser`'s stale CDP runtime-file cleanup. Before changing the owned browser backend, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` at local commit `3bb1d43`, where `auto_connect_cdp` reads `DevToolsActivePort`, tries to resolve the live CDP endpoint, and removes the file when the port is dead so future discovery skips stale state.

`OwnedBrowserAutomationBackend` now removes `DevToolsActivePort` from an owned/dedicated profile when `connect_existing_profile_browser` cannot connect through the exact WebSocket path or the `/json/version`, `/json/list`, and direct `/devtools/browser` discovery fallbacks. Non-CDP errors still propagate normally; only a dead/unavailable endpoint is treated as stale attach state.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test existing_profile_attach_removes_stale_devtools_active_port`
- `cargo test`

Confidence: High for dead-port stale-file cleanup. The behavior is source-backed and covered by a deterministic closed-port test; broader current-tab discovery and real-profile attach UX remain separate I19e work.

### D118: I19e adds silent Chrome startup diagnostics

The next startup-classification slice ports `agent-browser`'s no-stderr Chrome launch hint. Before changing the owned browser backend, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` at local commit `3bb1d43`, where `chrome_launch_error` adds an explicit no-stderr diagnostic and sandbox hint when Chrome exits before reporting a DevTools URL without producing stderr lines.

`OwnedBrowserAutomationBackend` now appends a no-stderr startup hint when Chrome exits or times out before CDP startup and the captured stderr file is empty. Existing profile-lock `requires_user_action` classification, relevant stderr lines, and sandbox/namespace hints still take precedence when diagnostic output exists.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test chrome_startup_error_adds_silent_exit_hint_without_stderr`
- `cargo test`

Confidence: High for silent-startup classification. The behavior is source-backed and covered at the classifier boundary; platform-specific real Chrome crashes still need opt-in smoke coverage.

### D119: I19d removes generic overlays before owned extraction

The next extraction-cleanup slice ports behavior that `aget` currently requested from Crawl4AI through `scripts/crawl4ai_extract.py`: `CrawlerRunConfig(remove_overlay_elements=True)`. Before changing the owned extractor, I19d re-inspected `references/repos/crawl4ai/crawl4ai/async_crawler_strategy.py` and `references/repos/crawl4ai/crawl4ai/js_snippet/remove_overlay_elements.js` at local commit `1debe5f`. Crawl4AI removes generic popup/modal/cookie overlay elements before capturing HTML; the JS snippet includes generic close-button, cookie-banner/consent, newsletter/subscribe, popup/modal/overlay/dialog, and dialog-role selectors.

`OwnedExtractorBackend` now applies the same generic selector cleanup before selector/exclusion extraction and before HTML/markdown/text serialization. This is intentionally generic and not site-specific: it removes DOM elements matching broad overlay/modal/cookie/dialog patterns, but it does not add built-in site names or paywall/login handling.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test --test mock_site_cli homegrown_extractor_backend_covers_static_http_parity_slice`
- `cargo test`

Confidence: High for generic selector-backed overlay removal. The behavior is source-backed and covered by a deterministic fixture that places cookie-banner and dialog-role elements inside the selected `<main>` and verifies both text and HTML outputs remove them. Style/z-index-based overlay removal from Crawl4AI's browser JS remains a rendered-page parity follow-up.

### D120: I19d removes rendered style overlays before CDP HTML capture

The next rendered-page cleanup slice ports the style/computed-layout side of Crawl4AI's `remove_overlay_elements` behavior. Before changing the owned CDP renderer, I19d re-inspected `references/repos/crawl4ai/crawl4ai/js_snippet/remove_overlay_elements.js` at local commit `1debe5f`. The upstream snippet clicks generic close/dismiss buttons, removes visible high-z-index/fixed/absolute overlay-like elements, removes elements matching generic popup/modal/cookie/dialog selectors, removes fixed/sticky elements, and resets body modal padding/overflow before HTML capture.

`OwnedExtractorBackend` now asks the owned CDP renderer to run a generic overlay cleanup script after navigation/waits/render-settle and before reading `document.documentElement.outerHTML`. The script is intentionally generic: it includes broad close/cookie/newsletter/popup/modal/overlay/dialog selectors plus computed style checks for high z-index, fixed/absolute positioning, overlay-like size/background/opacity, and fixed/sticky chrome. Cleanup failure is reported as an extraction warning rather than failing the whole fetch, so an overlay-cleanup regression does not turn an otherwise fetchable page into a hard error.

Validation:

- `cargo fmt --check`
- `cargo test rendered_overlay_cleanup_expression_uses_generic_crawl4ai_rules`
- `cargo test --test mock_site_cli owned_extractor_backend_removes_rendered_style_overlays_with_chrome -- --ignored`
- `git diff --check`
- `cargo test`

Confidence: High for rendered style-overlay cleanup on this machine. The behavior is source-backed, the script contract is covered by a deterministic unit test, and a local Chrome ignored smoke proves a style-only fixed overlay is removed before text extraction. Broader Crawl4AI-quality markdown/readability and richer rendered-readiness heuristics remain separate I19d work.
