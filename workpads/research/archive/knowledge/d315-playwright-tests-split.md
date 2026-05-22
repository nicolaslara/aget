# D315: Playwright Session Test Split

Date: 2026-05-22

## Decision

Split `src/session/playwright/tests.rs` into smaller behavior-owned child modules while preserving the existing parent test route.

## Boundary

- `src/session/playwright/tests.rs` now only routes test modules.
- `src/session/playwright/tests/playwright_state.rs` covers direct Playwright storage-state composition, cookie/origin deduplication, conflicts, and sessionStorage mapping.
- `src/session/playwright/tests/compose_session.rs` covers composed `Session` provenance, scope deduplication, storage merging, cookie identity normalization, and conflict redaction.
- `src/session/playwright/tests/state_file.rs` covers temp Playwright state file cleanup and private Unix permissions.
- `src/session/playwright/tests/support.rs` owns shared session/cookie/origin fixtures.

## Safety And Compatibility Notes

- This is a mechanical test decomposition only; session runtime behavior is unchanged.
- The existing `session::playwright` test module route remains in place.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.
- The parent test route dropped from 325 lines to 4 lines; all child modules are currently under 150 lines.

## Validation

- `cargo fmt`
- `cargo test session::playwright --lib`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
