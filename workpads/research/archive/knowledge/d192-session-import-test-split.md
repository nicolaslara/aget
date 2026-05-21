# D192: Session Import CLI Test Split

The `aget session import` CLI tests were mechanically split by behavior while preserving the parent `tests/session_cli.rs` integration test route.

Resulting boundaries:

- `tests/session_cli/imports.rs`: import-test module router.
- `tests/session_cli/imports/chrome_command.rs`: command-backed Chrome import, missing command backend, profile-lock classification, and malformed raw-state cleanup coverage.
- `tests/session_cli/imports/browser.rs`: generic `session import browser` Chrome import and unsupported-browser validation coverage.
- `tests/session_cli/imports/owned_chrome.rs`: owned Chrome startup classification coverage for profile-in-use and sandbox startup hints.
- `tests/session_cli/imports/real_smoke.rs`: ignored real-browser Chrome profile import smoke coverage.

The split did not intentionally change import command assertions, fixtures, backend selection, saved session checks, redaction checks, or cleanup expectations.

Validation:

- `cargo fmt --check`
- `cargo test --test session_cli imports::`
- `cargo test`
- `git diff --check`
