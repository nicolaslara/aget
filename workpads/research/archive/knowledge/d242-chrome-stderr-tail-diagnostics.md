# D242: Chrome Stderr Tail Diagnostics

Date: 2026-05-22

Source inspected:

- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`: `wait_for_ws_url_until` and `chrome_launch_error`.

Decision:

- Align owned generic Chrome startup diagnostics with agent-browser's bounded fallback of the last five stderr lines when no classified error keywords are present.
- Preserve existing owned classifications:
  - profile/session lock stderr still becomes `requires_user_action`;
  - sandbox/namespace stderr still receives the `AGET_CHROME_COMMAND` sandbox hint;
  - empty stderr still receives the silent Chrome startup hint.

Boundary:

- No agent-browser code was copied. The change only adjusts the owned diagnostic tail length from three generic lines to five.
- This does not close all I19e startup parity; real-profile/keychain smoke execution and broader startup/error classification remain open.

Validation:

- `cargo test browser_cdp::tests::discovery`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
