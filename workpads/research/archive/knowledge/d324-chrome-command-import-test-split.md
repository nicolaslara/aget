# D324: Chrome Command Import Test Split

## Decision

Keep `tests/session_cli/imports/chrome_command.rs` as the command-backed Chrome import test route and move the existing scenarios into child modules:

- `chrome_command/success.rs`: successful filtered Chrome import, inspect redaction, backend command sequence, and raw-state cleanup.
- `chrome_command/success_assertions.rs`: saved-session, inspect-redaction, and backend-cleanup assertions for the success scenario.
- `chrome_command/missing_backend.rs`: missing compatibility command reports `backend_unavailable`.
- `chrome_command/profile_lock.rs`: profile-lock classification reports `requires_user_action`.
- `chrome_command/malformed_state.rs`: malformed backend state closes the temporary session and removes raw state.

## Rationale

The split is mechanical and preserves the compatibility adapter coverage while reducing another large integration-test file. The behavior remains command-backed Chrome import coverage, not a runtime session import change.

## Validation

- `cargo test --test session_cli chrome_command`
