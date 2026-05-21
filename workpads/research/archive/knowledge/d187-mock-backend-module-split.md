# D187: Mock Backend Support Binary Module Split

The checked-in mock extractor backend used by CLI and mock-site integration tests was mechanically split to reduce the size of the standalone test support binary while preserving its Cargo binary entry path.

Resulting boundaries:

- `tests/support/bin/aget_mock_backend.rs`: binary root, descendant-marker mode, setup, and behavior dispatch call.
- `tests/support/bin/aget_mock_backend/args.rs`: mock backend CLI argument parsing.
- `tests/support/bin/aget_mock_backend/behavior.rs`: configured mock behavior execution.
- `tests/support/bin/aget_mock_backend/config.rs`: JSON config loading, expectation checks, state reading, and structured validation errors.
- `tests/support/bin/aget_mock_backend/http.rs`: loopback HTTP fetching, redirect handling, state-derived headers, and minimal HTML text helpers.
- `tests/support/bin/aget_mock_backend/output.rs`: backend result JSON printing.

The first focused run exposed that the mock-tool Cargo package compiles the binary through a relative `path`, so module discovery uses `tests/support/bin/` as the base. The root binary now uses explicit `#[path = "aget_mock_backend/..."]` module paths.

Validation:

- `cargo fmt --check`
- `cargo test --test get_cli --test mock_site_cli --test mock_site_sessions --test mock_site_docs_contract`
- `cargo test`
- `git diff --check`
