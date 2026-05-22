# D341: CDP readiness Runtime.evaluate exceptions

## Decision

Owned Chrome/CDP readiness and preprocessing helpers now treat `Runtime.evaluate` `exceptionDetails` as stable extraction failures instead of interpreting the response as false, empty, or successful.

## Source Evidence

- agent-browser snapshot: `references/repos/agent-browser` at `3bb1d43f8bb16444596365496f78395da8f1e6b7`.
- `cli/src/native/browser.rs` returns an evaluation error when `Runtime.evaluate` includes `exception_details`, preferring `exception.description` and falling back to `details.text`.
- D340 applied this boundary to owned string capture. D341 extends the same source-backed behavior to readiness and preprocessing evaluations.

## Owned Boundary

- Selector waits and image waits fail immediately when CDP reports evaluation exceptions instead of waiting until timeout or treating the page as not ready.
- Full-page scanning, rendered overlay cleanup, and iframe processing now surface evaluation exceptions through their helper result.
- Optional preprocessing steps that already continue after helper errors keep that continuation behavior by recording warnings at the existing call sites.
- Missing or non-exception evaluation values still follow their existing path-specific behavior.

## Validation

- `cargo test browser_cdp_wait_for_selector_reports_runtime_evaluation_exception`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
