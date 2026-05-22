# D316: Mock-Site Session Test Split

Date: 2026-05-22

## Decision

Split `tests/mock_site_sessions.rs` into smaller behavior-owned child modules while preserving the existing integration-test route.

## Boundary

- `tests/mock_site_sessions.rs` now only routes shared support plus behavior modules.
- `tests/mock_site_sessions/replay.rs` covers cookie-backed and storage-backed session replay against the mock site.
- `tests/mock_site_sessions/auth_states.rs` covers unauthenticated, expired-session, and logout responses.
- `tests/mock_site_sessions/chrome_import.rs` covers mocked Chrome session import followed by protected-page replay.
- `tests/mock_site_sessions/login_bootstrap.rs` covers mocked login start/finish followed by protected-page replay.

## Safety And Compatibility Notes

- This is a mechanical test decomposition only; CLI, session, mock-site, and backend behavior are unchanged.
- The root integration-test crate keeps `mod support` and uses explicit `#[path = "..."]` module routes, matching the existing split integration-test pattern.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.
- The parent test route dropped from 318 lines to 10 lines; all child modules are currently 104 lines or less.

## Validation

- `cargo fmt`
- `cargo test --test mock_site_sessions`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
