# D354: CDP Transport Test Split

## Decision

CDP client WebSocket transport tests now live in their own behavior-owned module instead of sharing the setup/attach-domain test file.

## Implementation

- `src/browser_cdp/tests/chrome/cdp_client/transport.rs` now owns CDP WebSocket config, keepalive, dialog auto-handling, binary-frame, invalid-binary-frame, and malformed-frame tests.
- `src/browser_cdp/tests/chrome/cdp_client/setup.rs` now stays focused on direct page connections, existing-page attachment setup, and page-domain enablement.
- `src/browser_cdp/tests/chrome/cdp_client.rs` registers the new `transport` module.
- No production code changed and no behavior was intentionally changed.
- `setup.rs` dropped from 400 lines to 144 lines; the new focused transport module is 265 lines.

## Validation

- `cargo test browser_cdp_auto_accepts_alert_dialogs_while_waiting_for_response`
- `cargo test browser_cdp_attach_existing_page_enables_discovery_before_target_list`
- `cargo test browser_cdp_enable_page_domains_auto_attaches_subtargets`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

## Follow-Up

- Keep using small mechanical split commits when test files grow around newly added behavior.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.
