# D256: Main Session Import/Login Split

## Decision

Split binary session import and login command handlers out of `src/main_session/mod.rs`.

## Boundary

- `src/main_session/mod.rs` remains the binary-facing `run_session` dispatcher and keeps list, authorize, inspect, delete, and compose routing.
- `src/main_session/import_command.rs` owns `aget session import` output and dispatch for cmux, browser-profile, and Chrome-profile import sources.
- `src/main_session/login_command.rs` owns `aget session login` start/finish/cancel output, JSON envelope shaping, and OAuth warning text.
- Existing `command_name`, `envelope`, `inspect`, and `profile` helper modules remain unchanged.

## Compatibility

The split is mechanical. Command names, JSON envelope fields, plain output strings, warnings, unsupported-browser usage errors, and error-response wrapping remain routed through the same dispatcher.

## Validation

- `cargo test --test session_cli imports`
- `cargo test --test session_cli login`

Standard validation for the stable point is recorded in the task status/commit that includes this decision.
