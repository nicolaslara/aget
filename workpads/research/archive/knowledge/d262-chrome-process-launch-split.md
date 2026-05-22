# D262: Chrome Process Launch Split

## Decision

Split Chrome launch command helpers out of `src/browser_cdp/chrome_process.rs`.

## Boundary

- `src/browser_cdp/chrome_process.rs` keeps `ChromeProcess` temp/profile/login launch entrypoints, launch retry orchestration, startup diagnostics classification, child process lifecycle, detach/wait-or-kill behavior, and owned temp profile cleanup.
- `src/browser_cdp/chrome_process/launch.rs` owns Chrome launch command construction, launch flags, process-group configuration, Chrome binary discovery, platform candidate lookup, PATH lookup, and temp owned-profile directory creation.

## Compatibility

The split is mechanical. Existing render/import/login callers continue to use `ChromeProcess`, and existing launch-argument tests continue to exercise command construction through the `chrome_process` module.

## Validation

- `cargo test browser_cdp::chrome_process::tests`
- `cargo test browser_cdp::tests::chrome::chrome_process`

Standard validation for the stable point is recorded in the task status/commit that includes this decision.
