# D191: Aget Facade/Backend API Test Split

The direct `Aget` facade/backend integration tests were mechanically split by behavior while preserving the `tests/aget_api.rs` integration test target.

Resulting boundaries:

- `tests/aget_api.rs`: integration-test module router.
- `tests/aget_api/support.rs`: in-memory session store, inspecting/failing/authorization extractors, test browser backend, and session/pending-login helpers.
- `tests/aget_api/extraction.rs`: custom session-store/extractor wiring and browser fallback after extractor failure.
- `tests/aget_api/authorization.rs`: Chrome authorization baseline/import/verification behavior and `requires_user_action` preservation.
- `tests/aget_api/session_backend.rs`: static browser backend login finish, Chrome import, and start/cancel behavior through custom session store.
- `tests/aget_api/browser_backend.rs`: default `AgetBrowserBackend` missing-profile import and pending-login cancellation behavior.

The split did not intentionally change backend trait contracts, public facade calls, assertions, or default backend behavior.

Validation:

- `cargo fmt --check`
- `cargo test --test aget_api`
- `cargo test`
- `git diff --check`
