# D284: CDP Navigation Wait Timeout Messages

Decision: owned CDP navigation waits now report the specific wait condition that timed out.

Source inspection:

- `references/repos/agent-browser/cli/src/native/browser.rs`: `wait_for_lifecycle` returns `Timeout waiting for {event_name}` for lifecycle waits.
- `references/repos/agent-browser/cli/src/native/browser.rs`: `poll_network_idle` maps the overall timeout to `Timeout waiting for networkidle`.

Implementation boundary:

- Owned lifecycle waits now return `owned browser fallback timed out waiting for load` or `owned browser fallback timed out waiting for domcontentloaded` instead of the generic Chrome-CDP polling timeout.
- Owned `networkidle` waits now return `owned browser fallback timed out waiting for networkidle`.
- The timeout error code remains `timeout`, and the wait behavior remains unchanged.
- This is a bounded error-classification parity slice; it does not add new wait conditions or broader readiness heuristics.

Validation:

- `cargo test browser_cdp::tests::chrome::cdp_client::navigation`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
