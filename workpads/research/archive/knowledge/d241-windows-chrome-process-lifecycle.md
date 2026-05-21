# D241: Windows Chrome Process Lifecycle Hooks

## Source Evidence

- `references/repos/agent-browser/cli/src/connection.rs`: Windows daemon startup uses `CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS`, and stale daemon cleanup uses `taskkill /PID <pid> /F`.
- `references/repos/agent-browser/cli/src/main.rs`: Windows dashboard startup uses the same detached process-group flags, and close-all fallback terminates unreachable Windows processes by pid.

## Decision

- Keep existing Unix Chrome process-group behavior unchanged.
- Apply the same Windows detached process-group flags to owned Chrome launches.
- Replace owned Chrome/login cleanup's non-Unix no-op termination path with Windows pid-based `taskkill` before falling back to `Child::kill`.
- For detached owned-login cleanup on Windows, treat the stored pid as the ownership proof, matching agent-browser's pid-based cleanup model.

## Validation

- `cargo test browser_cdp::process::tests -- --nocapture`
- `cargo test browser_cdp::tests::chrome::chrome_process::chrome_launch_retries_after_early_startup_exit -- --nocapture`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`
