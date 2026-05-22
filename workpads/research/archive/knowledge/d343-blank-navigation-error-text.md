# D343: Blank-response storage navigation errorText

## Decision

Owned Chrome/CDP blank-response navigation now reports `Page.navigate` `errorText` as a stable extraction failure.

## Source Evidence

- agent-browser snapshot: `references/repos/agent-browser` at `3bb1d43f8bb16444596365496f78395da8f1e6b7`.
- `cli/src/native/state.rs` uses a temporary target, enables `Fetch`, navigates each candidate origin, fulfills intercepted requests with blank HTML, and waits for `Page.loadEventFired` before collecting origin storage.
- The owned implementation uses the same blank-response concept for storage export through `navigate_with_blank_response`.

## Owned Boundary

- `navigate_with_blank_response` now reuses the owned `Page.navigate` response parser used by normal page navigation.
- CDP command errors and `result.errorText` now return stable `ExtractionFailed` messages instead of waiting for a load event that will not arrive.
- Normal blank fulfillment through `Fetch.requestPaused` and success on `Page.loadEventFired` are unchanged.

## Validation

- `cargo test browser_cdp_blank_response_navigation_reports_error_text`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
