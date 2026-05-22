# D287: Chrome Early-Exit Startup Message

Decision: owned Chrome/CDP startup now reports early Chrome exits with an explicit exit code and missing-`DevToolsActivePort` condition.

Source inspection:

- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`: `wait_for_devtools_active_port_until` returns `Chrome exited early (exit code: X) without writing DevToolsActivePort` when Chrome exits before writing the startup port file.
- `references/repos/agent-browser/cli/src/native/cdp/chrome.rs`: the surrounding launch code preserves that message alongside Chrome stderr diagnostics through the retry path.

Implementation boundary:

- `wait_for_devtools_active_port` now reports `owned browser fallback Chrome exited early (exit code: X) without writing DevToolsActivePort`.
- The stable error code remains `backend_unavailable`.
- Existing stderr classification, retry behavior, `DevTools listening on ...` stderr fallback, and port-file discovery behavior are unchanged.
- This is a bounded startup-diagnostic parity slice; it does not change Chrome process launch flags, timeout policy, or profile import behavior.

Validation:

- `cargo test browser_cdp::tests::discovery::wait_for_devtools_active_port_reports_agent_browser_style_early_exit_code --lib`
- `cargo test browser_cdp::tests::discovery --lib`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
