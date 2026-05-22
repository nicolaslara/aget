# D323: Playwright Compose Implementation Split

## Decision

Keep `session::playwright::{compose_playwright_state, compose_session}` exported from the same public module while splitting the implementation under `src/session/playwright/compose/`:

- `cookies.rs`: cookie identity, normalization, and equality helpers.
- `storage.rs`: local/session storage merge conflict helpers and composed-origin state.
- `state.rs`: Playwright storage-state composition.
- `session.rs`: persisted composed-session construction.

## Rationale

The split is mechanical and preserves the session composition behavior that protects replay scope, provenance, and conflict reporting. It reduces the largest remaining session implementation file while keeping the public session/playwright interface stable for callers.

## Validation

- `cargo test session::playwright --lib`
