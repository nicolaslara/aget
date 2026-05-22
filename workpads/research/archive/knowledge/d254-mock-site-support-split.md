# D254: Mock Site Support Split

## Decision

Split shared mock-site support server helpers out of `tests/support/mock_site.rs`.

## Boundary

- `tests/support/mock_site.rs` keeps the public test-support API: `MockSite`, `MockSiteBuilder`, `MockResponse`, request recording, lifecycle/shutdown, and header/cookie assertion helpers.
- `tests/support/mock_site/protocol.rs` owns raw HTTP request parsing, cookie-header matching, and HTTP response formatting.
- `tests/support/mock_site/routes.rs` owns built-in default mock routes for public, protected, storage, delayed, redirect, login, and logout scenarios.

## Compatibility

The split is mechanical. Existing imports such as `support::mock_site::{MockResponse, MockSite, MockSiteBuilder}` remain compatible, custom route behavior is unchanged, and default route strings/statuses/headers stay in test support.

## Validation

- `cargo test --test mock_site_cli`
- `cargo test --test mock_site_browser`
- `cargo test --test mock_site_sessions`
- `cargo test --test mock_site_docs_contract`

Standard validation for the stable point is recorded in the task status/commit that includes this decision.
