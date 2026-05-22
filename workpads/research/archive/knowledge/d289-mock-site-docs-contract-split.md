# D289: Mock-Site Docs Contract Test Split

Decision: mock-site docs-contract integration tests now route through smaller behavior-focused modules.

Implementation boundary:

- `tests/mock_site_docs_contract.rs` is now a module index plus shared test support.
- `tests/mock_site_docs_contract/public_get.rs` owns the public get JSON/artifact contract.
- `tests/mock_site_docs_contract/output.rs` owns output file, warning, and limit contract coverage.
- `tests/mock_site_docs_contract/custom_site.rs` owns custom-route extraction fixture coverage.
- `tests/mock_site_docs_contract/session_replay.rs` owns session compose/replay and replay-scope rejection contract coverage.
- `tests/mock_site_docs_contract/session_lifecycle.rs` owns session login/list/inspect/fetch/delete contract coverage.
- Production behavior and assertions are unchanged.
- `workpads/research/tasks.md` was not compacted.

Validation:

- `cargo test --test mock_site_docs_contract`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
