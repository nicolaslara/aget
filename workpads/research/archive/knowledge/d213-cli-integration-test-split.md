# D213: CLI Integration Test Split

Date: 2026-05-21

Context:

- After D212, `tests/cli.rs` was the largest remaining Rust integration-test entrypoint in the tracked-file scan.
- The user wants large-file ergonomics improved while preserving `workpads/research/tasks.md` as the full planned-task source of truth.

Decision:

- Keep `tests/cli.rs` as the stable `cargo test --test cli` integration-test entrypoint.
- Move CLI integration tests into focused child modules:
  - `tests/cli/help.rs`: help output, clap parse errors, and structured parse-error envelopes.
  - `tests/cli/get.rs`: top-level URL alias execution and JSON envelope behavior.
  - `tests/cli/current_tab.rs`: current-tab consent and mock-CDP behavior.
  - `tests/cli/support.rs`: local HTTP server, mock current-tab CDP server, and mock backend tool build helpers.
- Use explicit `#[path = "cli/..."]` module routes from `tests/cli.rs` so integration-test module discovery does not collide with neighboring `tests/support` paths.
- Preserve existing assertions and command arguments.

Validation:

- `cargo fmt --check`
- `cargo test --test cli`
- `cargo test`
- `git diff --check`
