# D244: Chrome Launch Stability Flags

Date: 2026-05-22

Source inspected:

- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`: `build_chrome_args`.

Decision:

- Add the generic Chrome launch flags that agent-browser uses to reduce background work and startup noise:
  - `--disable-hang-monitor`
  - `--disable-prompt-on-repost`
  - `--enable-features=NetworkService,NetworkServiceInProcess`
  - `--metrics-recording-only`
- Preserve owned `aget` behavior for explicit user-data-dir/profile selection, mock-vs-real keychain, headless mode, Linux sandbox/dev-shm flags, Windows process-group flags, and headed login startup URLs.

Boundary:

- No agent-browser code was copied. The slice ports the observed launch-argument contract into the owned Chrome launcher.
- This does not close all I19e browser parity; real-profile/keychain smoke execution and broader rendered JavaScript parity remain open.

Validation:

- `cargo test chrome_launch_args_include_agent_browser_stability_flags`
- `cargo test browser_cdp::tests::chrome::chrome_process`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
