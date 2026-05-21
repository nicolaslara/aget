# D210: Consent-Gated Current Tab

Date: 2026-05-21

Source inspected:

- `references/repos/agent-browser/cli/src/native/browser.rs`: external CDP connection setup, current page target discovery/attachment, and external-browser close behavior.
- Reuses D208/D209 source inspection for CDP discovery and current-page capture.

Decision:

- Expose current-tab extraction through public `Aget::current_tab` and `aget current-tab`.
- Require explicit local endpoint ownership and consent: callers provide `--cdp-port` and `--allow-private-content`.
- Reuse owned browser/CDP current-tab rendering plus owned HTML-to-content conversion.
- Return the standard `GetSuccess` and JSON envelope shape so current-tab output has content formats, selectors/exclusions, CSS waits, limits, artifacts, warnings, and timing.
- Mark current-tab results sensitive by default, so JSON `--inline-content auto` omits `data.content`.

Boundaries:

- No ambient profile or port scanning.
- No site-specific login/paywall advice.
- No navigation, page creation, page close, or browser close.
- Command-backed `agent-browser` compatibility does not implement this new current-tab path; the public command uses the owned backend.

Validation:

- `cargo fmt --check`
- `cargo test cli::tests::parses_current_tab_command`
- `cargo test --test cli current_tab`
- `cargo test --test aget_api aget_current_tab_api`
- `cargo test`
- `git diff --check`
