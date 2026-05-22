# D253: Session CLI Root Test Split

## Decision

Split the remaining root `tests/session_cli.rs` integration tests into behavior-owned modules.

## Boundary

- `tests/session_cli.rs` is now a 15-line integration target router for authorize, compose, imports, cmux imports, inspect, list, login, and shared support.
- `tests/session_cli/list.rs` owns list/delete and list JSON shape coverage.
- `tests/session_cli/compose.rs` owns composed-session persistence, target validation, existing-session protection, and conflict-redaction coverage.
- `tests/session_cli/inspect.rs` owns composed-session inspect redaction/provenance coverage.

## Compatibility

The split is mechanical. Test target name, test function names, command arguments, assertions, helper usage, and ignored real-smoke routing are preserved.

## Validation

- `cargo test --test session_cli`

Standard validation for the stable point is recorded in the task status/commit that includes this decision.
