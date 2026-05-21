# D190: Session-Backed Get CLI Test Split

The session-backed `aget get` integration tests were mechanically split by behavior while preserving the parent `tests/get_cli.rs` route and existing assertions.

Resulting boundaries:

- `tests/get_cli/session.rs`: module router kept at the existing parent path.
- `tests/get_cli/session/replay.rs`: named session replay, replay-scope blocking, repeated session composition, app/provider cookie flow, and sensitive output marking.
- `tests/get_cli/session/fallback.rs`: session-backed backend failure redaction, agent-browser fallback, unauthenticated no-fallback behavior, and fallback close-failure handling.
- `tests/get_cli/session/real_smoke.rs`: ignored/manual real Crawl4AI cookie replay smoke.

The split did not intentionally change authenticated get behavior, mock setup, assertions, ignored-test requirements, or support helpers.

Validation:

- `cargo fmt --check`
- `cargo test --test get_cli`
- `cargo test`
- `git diff --check`
