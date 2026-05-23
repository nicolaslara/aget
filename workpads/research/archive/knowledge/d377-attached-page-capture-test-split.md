# D377: Attached Page Capture Test Split

Decision: split the oversized attached-page capture CDP test module into
behavior-focused child modules without changing test behavior.

Boundary after the split:

- `src/browser_cdp/tests/chrome/cdp_client/attached_page/capture/mod.rs`
  remains the parent route for attached-page capture coverage.
- `capture/basic.rs` covers reading the current URL and HTML from an already
  attached page without navigation.
- `capture/images.rs` covers the boundary that `wait_for_timeout` does not
  shorten image readiness polling.
- `capture/runtime_error.rs` covers surfacing `Runtime.evaluate` exception
  details from attached-page capture.

The split is mechanical. Test assertions, mock CDP request sequencing, and test
names are preserved.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
