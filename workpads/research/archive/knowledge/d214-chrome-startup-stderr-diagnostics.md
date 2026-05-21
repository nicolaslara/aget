# D214: Chrome Startup Stderr Diagnostics

Date: 2026-05-21

Source inspected:

- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`: `try_launch_chrome`, `wait_for_ws_url_until`, `chrome_launch_error`, and launch-error unit tests.

Decision:

- Adapt the source-backed diagnostic behavior that generic Chrome launch stderr should be explicitly labeled as recent Chrome stderr.
- Preserve existing owned classifications:
  - sandbox/namespace stderr keeps the `AGET_CHROME_COMMAND` sandbox hint;
  - silent startup failures keep the no-stderr startup hint;
  - profile-in-use and existing-session failures still classify as `requires_user_action`.
- Keep owned diagnostics bounded to the last three generic stderr lines so failures stay readable and do not dump large Chrome logs.

Boundary:

- No agent-browser code was copied; this ports the observed diagnostic contract into the owned `browser_cdp` startup classifier.
- This does not close all I19e startup/lifecycle parity; real-browser smoke coverage and cross-platform lifecycle behavior remain open.

Validation:

- `cargo fmt --check`
- `cargo test browser_cdp::tests::discovery`
- `cargo test`
- `git diff --check`
