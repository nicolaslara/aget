# D189: Get CLI Test Split

The non-session `aget get` CLI integration tests were mechanically split by behavior while preserving the existing session-backed get coverage module and shared support route.

Resulting boundaries:

- `tests/get_cli.rs`: integration-test module router and shared support module route.
- `tests/get_cli/success.rs`: public success, artifact, inline-content, and command environment behavior.
- `tests/get_cli/output.rs`: output options, content limits, JSON-format truncation, and sanitized backend stdout artifact behavior.
- `tests/get_cli/validation.rs`: Crawl4AI compatibility option validation and JavaScript wait rejection coverage.
- `tests/get_cli/failure.rs`: timeout, noisy/stdout backend output, nonzero/malformed backend output, structured failure, process cleanup, and missing backend coverage.
- `tests/get_cli/session.rs`: existing session-backed get coverage, unchanged in this slice.

The split did not intentionally change assertions, mock behavior, ignored-test requirements, or support helpers.

Validation:

- `cargo fmt --check`
- `cargo test --test get_cli`
- `cargo test`
- `git diff --check`
