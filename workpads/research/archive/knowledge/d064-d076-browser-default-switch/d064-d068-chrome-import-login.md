# D64-D68: Chrome Import And Login Lifecycle

### D64: I19e adds bounded owned Chrome profile-path import

Before porting this import slice, I19e inspected the `agent-browser` state/profile paths that back the current command adapter:

- `references/repos/agent-browser/cli/src/native/state.rs` exports Playwright-style storage state by reading cookies through CDP and collecting local/session storage from the current page plus known origins.
- `references/repos/agent-browser/cli/src/native/cookies.rs` uses `Network.getAllCookies` and URL-scoped cookie reads as the browser-state bridge.
- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs` launches Chrome with a user-data-dir, removes stale `DevToolsActivePort` files before launch, copies named Chrome profiles to temporary directories, and uses the real keychain only for copied named-profile imports.

`OwnedBrowserAutomationBackend::import_chrome` now has a bounded owned path for explicit profile directories. It launches local Chrome against the supplied user-data-dir path, exports cookies and localStorage through the in-process CDP client, filters the resulting Playwright-style state through the same allowlist/provenance logic used for command-backed `agent-browser` state, and persists only scoped session material. LocalStorage collection uses request interception to load blank same-origin documents for allowed domains, avoiding real network requests while still reading origin storage.

This deliberately does not claim parity with named Chrome profiles such as `Default`. Named profile import still needs the higher-risk `agent-browser` behavior: resolving Chrome's user-data-dir, copying only the selected profile plus `Local State`, preserving macOS/OS keychain behavior where needed, diagnosing locked/running profiles, and cleaning copied profiles. For now, the owned backend returns a structured backend-unavailable error for named profiles and leaves the command-backed adapter as the supported path.

Validation:

- `cargo test browser_cdp`
- `cargo test session::chrome`
- `cargo test session::agent_browser`
- `cargo test --test aget_api owned_browser_backend_reports_named_profile_import_gap`
- `cargo test browser_cdp::tests::owned_chrome_import_exports_cookie_and_local_storage_from_profile -- --ignored` passed locally with system Chrome, proving a temporary user-data-dir profile can persist a cookie/localStorage pair and be re-imported through the owned CDP export path.

Confidence: Medium. The explicit profile-path slice is now owned and tested, but full `agent-browser` import parity remains open until named real-profile copy/keychain/lock behavior is ported.

### D65: I19e ports named Chrome profile resolution and copy setup

The next I19e Chrome-import slice ports the named-profile setup that `agent-browser --profile Default` depends on. The relevant source remains `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`, especially `get_chrome_user_data_dirs`, `find_chrome_user_data_dir`, `list_chrome_profiles`, `resolve_chrome_profile`, `copy_chrome_profile`, and `copy_dir_recursive`.

`OwnedBrowserAutomationBackend::import_chrome` now treats profile arguments without path separators as Chrome profile names. It finds a Chrome user-data directory with `Local State` (or `AGET_CHROME_USER_DATA_DIR` for deterministic tests), resolves the requested profile by exact directory, display name, or case-insensitive directory, copies `Local State` plus the selected profile subdirectory into a private temporary user-data-dir, skips large/cache/lock directories and files, launches Chrome with `--profile-directory=<resolved>`, exports scoped cookies/localStorage through CDP, and removes the temporary profile copy on drop. The copied-profile launch avoids the mock keychain flags so real Chrome profile imports have the same keychain shape as the upstream command adapter. Orphan sweeping now also removes stale `tmp/owned-chrome-import/aget-profile-*` copies.

This still needs a real logged-in profile smoke before claiming full parity with user Chrome `Default` imports. The deterministic tests cover profile resolution, ambiguous/missing profile errors, copy exclusions, private temp directory permissions, failure-to-save behavior, and a local Chrome smoke for the `--profile-directory` CDP export path. They do not prove macOS/OS keychain cookie decryption against the user's real Chrome profile or profile-lock classification for an actively running browser.

Validation:

- `cargo test session::chrome`
- `cargo test browser_cdp`
- `cargo test session::store::tests::orphan_sweep`
- `cargo test --test aget_api owned_browser_backend_does_not_save_failed_profile_path_import`
- `cargo test browser_cdp::tests::owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory -- --ignored` passed locally with system Chrome, proving the CDP export path works when Chrome is launched with a selected profile directory.

Confidence: Medium. The named-profile setup is now owned and deterministically covered, but real-profile auth/keychain and lock/error classification remain higher-risk I19e follow-ups.

### D66: I19e adds a first owned dedicated-profile login lifecycle

Before porting the login lifecycle, I19e inspected the `agent-browser` command and native paths that back the current `aget session login start|finish|cancel` flow:

- `references/repos/agent-browser/cli/src/commands.rs`: parses `open`, `state save <path>`, and `close`.
- `references/repos/agent-browser/cli/src/native/browser.rs`: sends `Browser.close` only for locally launched browsers, so external/current-browser connections are not shut down accidentally.
- `references/repos/agent-browser/cli/src/native/state.rs`: exports Playwright-style cookies and origin storage through CDP, including blank-response origin visits for storage collection.
- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`: launches Chrome with an explicit `--user-data-dir`, uses mock keychain flags by default, omits headless mode for visible browser flows, and reads `DevToolsActivePort` for CDP attachment.

`OwnedBrowserAutomationBackend` now implements `start_login`, `finish_login`, and `cancel_login` for dedicated `aget` profiles without shelling out to `agent-browser`. Start creates a private `tmp/owned-login/aget-<name>` profile by default, launches visible Chrome at the caller-provided HTTPS URL, records pending login metadata, and leaves Chrome running for user-driven auth. Finish connects to the running profile browser through `DevToolsActivePort` when available, exports cookies/localStorage for only the pending flow's allowed domains, closes the browser, filters the state through the existing session allowlist/provenance logic, and still returns `requires_user_action` when no scoped auth state is found. If the user already closed the browser, finish falls back to a headless launch against the same dedicated profile to export state. Cancel closes the running profile browser when reachable and cleans pending metadata plus tool-owned login profiles; custom profile paths are preserved.

This slice preserves the public login API and the existing `SessionSource::AgentBrowser` shape for compatibility, even though the implementation is now owned. It does not copy upstream code. The remaining parity gaps are real manual login smoke coverage, richer process diagnostics, Windows/profile-lock behavior, current-tab attach, and complete `requires_user_action` classification for all Chrome startup/export failures.

Validation:

- `cargo test session::login::tests`
- `cargo test --test aget_api owned_browser_backend_cancels_pending_login_without_agent_browser`
- `cargo test --test aget_api owned_browser_backend`
- `cargo test browser_cdp`
- `cargo test`
- `cargo fmt --check`
- `git diff --check`

Confidence: Medium-high. The deterministic tests cover ownership boundaries, cleanup, public backend wiring, and CDP payload helpers; the ignored headed Chrome smoke test now passes locally for disposable profile start/export/close. A real user-authorized site login has still not been run in this slice.

### D67: I19e sweeps stale owned-login profiles without deleting pending flows

The D66 lifecycle introduced `tmp/owned-login/aget-<name>` profile directories for owned dedicated login sessions. Startup orphan sweeping now includes this root, but only removes an `aget-<name>` profile when the matching `tmp/login-<name>.json` pending metadata file is absent and the profile is older than the sweep threshold. This keeps active or paused user login flows intact while still cleaning stale profiles left by crashes, manual file deletion, or failed handoffs.

Validation:

- `cargo test session::store::tests::orphan_sweep`
- `cargo test session::login::tests`
- `cargo test`
- `cargo fmt --check`
- `git diff --check`

Confidence: High for this cleanup slice. The behavior is narrow and deterministic; broader lifecycle/process parity remains covered by D66's open gaps.

### D68: I19e hardens owned-login process cleanup after headed Chrome smoke

The ignored headed Chrome smoke initially proved state export but exposed a lifecycle flaw: `Browser.close` could return while the detached visible Chrome process was still alive, and immediate profile removal could race or leave a disposable browser process behind. The owned login start path now records the launched browser PID in pending login metadata. Finish and cancel pass that PID into the CDP close path, wait for the process to exit after `Browser.close`, and only terminate the PID/process group as a fallback when the process command line still references the expected profile path.

This keeps process cleanup scoped to `aget`-launched dedicated login browsers. Command-backed `agent-browser` flows still have no PID in pending metadata and keep their existing close behavior. Pending metadata remains backward-compatible because `browser_pid` defaults to `None` when older pending files are read.

Validation:

- `cargo test browser_cdp::tests::owned_login_browser_exports_state_from_headed_profile_and_closes -- --ignored`
- `cargo test session::login::tests`
- `cargo test`
- `cargo fmt --check`
- `git diff --check`
- Manual process check after the ignored smoke: no remaining `login-profile`, `owned-login`, or `remote-debugging-port=0` test Chrome process.

Confidence: Medium-high. The specific detached-process leak is now covered by a real local Chrome smoke and guarded cleanup logic, but broader cross-platform process behavior still needs Windows/Linux verification before I19e can be considered complete.
