# D212: CLI Parser Test Split

Date: 2026-05-21

Context:

- After the major migration-module splits, `src/cli/tests.rs` was the largest source-side unit-test file in the tracked-file scan.
- The user asked to keep improving large-file ergonomics while not compacting `workpads/research/tasks.md`.

Decision:

- Keep `src/cli/tests.rs` as the stable `src/cli.rs` test-module route.
- Move CLI parser unit tests into focused child modules:
  - `src/cli/tests/global.rs`: top-level URL aliasing and global flag parsing.
  - `src/cli/tests/get.rs`: `aget get` parser assertions.
  - `src/cli/tests/current_tab.rs`: `aget current-tab` parser assertions.
  - `src/cli/tests/session.rs`: `aget session` parser assertions.
- Preserve existing parser assertions and behavior.

Validation:

- `cargo fmt --check`
- `cargo test cli::tests`
- `cargo test`
- `git diff --check`
