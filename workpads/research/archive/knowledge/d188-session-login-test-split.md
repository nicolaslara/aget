# D188: Session Login CLI Test Split

The session login CLI integration tests were mechanically split by behavior so future browser/login work does not require loading one large mixed test module.

Resulting boundaries:

- `tests/session_cli/login.rs`: module router kept at the existing parent path.
- `tests/session_cli/login/start.rs`: `aget session login start` CLI coverage.
- `tests/session_cli/login/finish.rs`: `aget session login finish` save, merge, failure, and provider-only coverage.
- `tests/session_cli/login/cancel.rs`: `aget session login cancel` cleanup and close-failure coverage.
- `tests/session_cli/login/direct.rs`: direct login lifecycle API coverage that bypasses the CLI.
- `tests/session_cli/login/real_smoke.rs`: ignored/manual HelloInterview login smoke.

The split preserved the parent `tests/session_cli.rs` route and did not intentionally change any session login behavior, assertions, or ignored-test requirements.

Validation:

- `cargo fmt --check`
- `cargo test --test session_cli`
- `cargo test`
- `git diff --check`
