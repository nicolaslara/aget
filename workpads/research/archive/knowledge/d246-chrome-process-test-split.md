# D246: Chrome Process Test Split

Date: 2026-05-22

Decision:

- Keep `src/browser_cdp/chrome_process.rs` focused on production Chrome launch, retry, binary discovery, and lifecycle cleanup behavior.
- Move deterministic launch-command unit tests into `src/browser_cdp/chrome_process/tests.rs`.
- Preserve private helper access with a nested test module instead of widening production API visibility.
- Leave `workpads/research/tasks.md` as the full planned-task backlog; this slice does not compact tasks.

Boundary:

- Runtime Chrome/CDP behavior is unchanged.
- The split only moves existing launch-argument coverage out of the production module.

Validation:

- `cargo test browser_cdp::chrome_process`
- `cargo fmt --check`
- `cargo test`
