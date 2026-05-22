# D342: CDP storage Runtime.evaluate exceptions

## Decision

Owned Chrome/CDP storage load and export now treat `Runtime.evaluate` `exceptionDetails` as stable extraction failures instead of treating storage writes as successful or storage reads as absent.

## Source Evidence

- agent-browser snapshot: `references/repos/agent-browser` at `3bb1d43f8bb16444596365496f78395da8f1e6b7`.
- `cli/src/native/storage.rs` evaluates storage JavaScript with `Runtime.evaluate` and returns `Storage error: ...` when CDP includes `exception_details`.
- `cli/src/native/state.rs` uses `Runtime.evaluate` to collect origin storage during export, making exception-aware handling relevant to owned load/export parity.

## Owned Boundary

- The shared owned CDP client now has one `Runtime.evaluate` exception helper used by rendered capture, readiness/preprocessing, and storage state code.
- Loading localStorage or sessionStorage fails with stable `ExtractionFailed` when CDP reports evaluation exceptions.
- Exporting origin storage fails with the same stable evaluation failure instead of silently skipping the affected origin.
- Existing non-exception behavior is preserved, including empty/missing storage reads being treated as no exported origin.

## Validation

- `cargo test browser_cdp_load_state_reports_storage_runtime_evaluation_exception`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
