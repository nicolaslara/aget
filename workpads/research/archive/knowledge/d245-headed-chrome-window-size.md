# D245: Headed Chrome Window Size

Date: 2026-05-22

Source inspected:

- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`: `build_chrome_args`.

Decision:

- Keep `--window-size=1280,720` on owned headless Chrome launches for rendered extraction.
- Omit the default `--window-size` on owned headed visible login Chrome launches, matching agent-browser's headed launch behavior.
- Preserve existing profile, keychain, Linux sandbox/dev-shm, Windows process-group, startup URL, and headless SwiftShader behavior.

Boundary:

- No agent-browser code was copied. The slice ports the observed launch-argument boundary into the owned Chrome launcher.
- This affects visible login browser launch arguments only; extraction rendering remains headless with the same default viewport.

Validation:

- `cargo test chrome_launch_args_include_default_window_size_only_when_headless`
- `cargo test chrome_launch_args_include_agent_browser_stability_flags`
- `cargo test browser_cdp::chrome_process`
- `cargo fmt --check`
- `cargo test`
