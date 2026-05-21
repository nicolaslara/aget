# D186: Binary Session Command Module Split

The binary-layer session command implementation was mechanically split so future CLI work does not require loading the full `aget session` dispatcher plus all helper subdomains.

Resulting boundaries:

- `src/main_session/mod.rs`: binary-facing `run_session` dispatcher and command output flow.
- `src/main_session/command_name.rs`: stable structured-envelope command names for session subcommands.
- `src/main_session/profile.rs`: browser/profile argument normalization and usage-error messages.
- `src/main_session/envelope.rs`: session authorization JSON envelope data.
- `src/main_session/inspect.rs`: inspect/redaction view structs and plain-output provenance suffix helpers.

The split preserved `main_session::run_session` as the only entry used by `src/main.rs`; no CLI names, JSON envelope fields, redaction behavior, or session backend calls were intentionally changed.

Validation:

- `cargo fmt --check`
- `cargo test --test session_cli`
- `cargo test`
- `git diff --check`
