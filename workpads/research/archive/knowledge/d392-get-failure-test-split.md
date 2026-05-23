# D392: Get Failure Test Split

Decision: split get CLI failure tests into behavior-focused modules without
changing failure coverage or assertion behavior.

Boundary after the split:

- `tests/get_cli/failure.rs` keeps the compact route used by `tests/get_cli.rs`.
- `failure/timeout.rs` owns timeout envelope/metadata and descendant process
  termination coverage.
- `failure/output.rs` owns noisy backend output and stdout-log-before-JSON
  success coverage.
- `failure/backend.rs` owns nonzero, malformed, and structured backend result
  failure coverage.
- `failure/missing.rs` owns missing backend classification and metadata
  coverage.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
