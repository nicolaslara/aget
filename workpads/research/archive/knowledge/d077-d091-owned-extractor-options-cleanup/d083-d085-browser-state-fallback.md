# D83-D85: Browser State And Fallback

### D83: I19e classifies Chrome profile-in-use startup as user action

The next owned browser/session parity slice tightens Chrome startup diagnostics for profile imports. Before changing the owned path, I19e re-inspected `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`, where `agent-browser` captures Chrome startup stderr, reports early `DevToolsActivePort` failures with stderr context, copies named profiles into temporary user-data-dir roots, and treats profile reuse/lock situations as launch failures that a caller can surface to the user.

`OwnedBrowserAutomationBackend` now captures Chrome stderr in a private temp file during CDP startup. If Chrome exits before writing `DevToolsActivePort` and stderr indicates a profile-in-use or singleton/lock condition, the owned backend returns `requires_user_action` instead of a generic backend failure. Non-user-action startup failures still keep their original error code but include relevant Chrome stderr lines for diagnosis. This improves owned Chrome/profile import parity without touching the user's running browser or adding ambient current-browser access.

Validation:

- `cargo fmt --check`
- `cargo test owned_session_import_chrome_classifies_profile_in_use_as_requires_user_action --test session_cli`
- `cargo test browser_cdp::tests::parses_devtools_active_port_file`
- `cargo test browser_cdp::tests::owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory -- --ignored`

Confidence: Medium-high. The new deterministic CLI test proves the classification and no-session-saved behavior with a fake Chrome executable. Real Chrome lock/keychain behavior still needs the ignored/manual smoke coverage tracked under I19e/I19h.

### D84: I19e covers scripted owned browser fallback rendering

The next I19e verification slice covers an `agent-browser` parity behavior without adding new production surface. The command-backed fallback path in `src/extraction.rs` loads composed state into a browser session, opens the requested URL, then reads body HTML/text from the rendered page. The upstream `agent-browser` navigation path in `references/repos/agent-browser/cli/src/native/browser.rs` waits for a CDP lifecycle event before callers request page content.

`tests/mock_site_cli.rs` now includes an ignored local-Chrome smoke proving the owned browser fallback renders a cookie-backed script-bearing page after the primary extractor fails. This specifically exercises the fallback path rather than the default owned primary extractor, and verifies the session cookie is replayed to the scripted page.

Validation:

- `cargo test --test mock_site_cli owned_browser_fallback_renders_cookie_backed_scripted_page_with_chrome -- --ignored`

Confidence: Medium-high for this parity slice. The smoke uses local Chrome and a deterministic local site, but broader SPA readiness, current-tab attach, and real logged-in profile/keychain behavior remain open.

### D85: I19e preserves owned browser sessionStorage state

The next I19e state-parity slice ports the sessionStorage behavior that was still only partially represented in `aget`. Before changing the owned path, I19e re-inspected `references/repos/agent-browser/cli/src/native/state.rs`, where `StorageState` includes per-origin `sessionStorage`, state export collects `sessionStorage` through `Runtime.evaluate`, and state load navigates to each origin before calling `sessionStorage.setItem(...)`.

`aget` now keeps sessionStorage as first-class scoped session state. `SessionOrigin` stores it beside localStorage, agent-browser/Playwright-state imports preserve it through the same allowlist filtering, composed state merges it with duplicate-key conflict checks, owned CDP fallback loading replays it after navigating to the origin, and CDP state export collects origins that contain either localStorage or sessionStorage. Live owned-login export now attaches to an existing non-internal page target before state export instead of creating a fresh tab, because sessionStorage is tab-scoped and would otherwise be lost. Normal session inspection and extraction redaction now treat sessionStorage values as credential-equivalent bearer material, while `--show-secrets` can still reveal them intentionally.

Validation:

- `cargo check`
- `cargo test session_storage`
- `cargo test --test session_cli session_import_chrome_saves_filtered_state_and_cleans_raw_file`
- `cargo test browser_cdp::tests::parses_origin_storage_runtime_value`
- `cargo test browser_cdp::tests::prefers_existing_non_internal_page_target`
- `cargo test browser_cdp::tests::owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory -- --ignored`
- `cargo test browser_cdp::tests::owned_login_browser_exports_state_from_headed_profile_and_closes -- --ignored`

Confidence: Medium-high. Deterministic tests cover import filtering, redaction, composition, JS expression quoting, target selection, and CDP result parsing. The local-Chrome ignored smokes cover persistent profile export plus live headed-login sessionStorage export, but current-tab attach and broader cross-platform browser-process behavior remain I19e follow-ups.
