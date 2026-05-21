# D215: Real Chrome Import Smoke

Date: 2026-05-21

Source inspected:

- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`: named Chrome profile handling resolves a display/directory name, copies `Local State` plus the selected profile directory to a temp user-data-dir, sets `use_real_keychain`, and launches Chrome with `--profile-directory=<resolved>`.
- `src/session/chrome/profile.rs` and `src/session/chrome/owned.rs`: owned `aget` mirrors the same boundary for named profiles, while explicit profile paths do not use the real keychain flag.

Decision:

- Strengthen the ignored/manual real Chrome import smoke from command-exit-only to persisted-session verification.
- Require both `AGET_REAL_BROWSER_PROFILE` and `AGET_REAL_BROWSER_DOMAIN`; no default domain is acceptable because this smoke is meant to prove approved scoped auth from a real logged-in profile.
- After `session import browser --browser chrome`, parse the JSON envelope, load the saved session through `SessionStore`, verify Chrome profile provenance, verify the requested cookie-domain allowlist, and assert nonempty scoped cookies or storage.
- Keep the test output limited to counts and structural assertions. It must not print cookie or storage values.

Boundary:

- This is still an ignored, opt-in local smoke. Without approved env vars it returns early, so CI can build the path without touching a real browser profile.
- Passing the no-env path proves the test builds and remains safely gated. It does not prove a local machine's real keychain/profile behavior until run with an approved profile and domain.
- I19e remains open for broader rendered JavaScript parity, manual real-profile/keychain smoke execution, cross-platform process lifecycle parity, and still-fuller startup/error classification.

Validation:

- `cargo fmt --check`
- `cargo test --test session_cli real_session_import_browser_chrome_profile -- --ignored`
- `cargo test`
- `git diff --check`
