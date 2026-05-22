# D340: CDP Runtime.evaluate exception details

## Decision

Owned Chrome/CDP rendered-page capture now treats `Runtime.evaluate` `exceptionDetails` as a stable extraction failure instead of silently converting the missing value to an empty string.

## Source Evidence

- agent-browser snapshot: `references/repos/agent-browser` at `3bb1d43f8bb16444596365496f78395da8f1e6b7`.
- `cli/src/native/browser.rs` sends `Runtime.evaluate` with `return_by_value: Some(true)` and `await_promise: Some(true)`.
- When CDP returns `exception_details`, agent-browser prefers `exception.description`, falls back to `details.text`, and returns `Evaluation error: ...` instead of returning a null/empty value.

## Owned Boundary

- `CdpClient::evaluate_string` now checks the CDP response for `exceptionDetails` before reading `result.value`.
- The stable owned error uses `ExtractionFailed` and includes the browser-provided exception description or text.
- Existing no-exception behavior is preserved: missing or non-string `result.value` still becomes an empty string.
- Both launched-page rendering and current-tab attachment share this helper, so the direct browser-session path and attached-page path remain compatible.

## Validation

- `cargo test browser_cdp_render_attached_page_reports_runtime_evaluation_exception`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
