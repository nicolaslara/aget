# D261: Chrome Profile Discovery Split

## Decision

Split Chrome profile discovery and name-resolution helpers out of `src/session/chrome/profile.rs`.

## Boundary

- `src/session/chrome/profile.rs` keeps owned Chrome profile preparation, explicit profile path validation, copied-profile lifetime, profile snapshot copying, copy exclusions, and private-directory helpers.
- `src/session/chrome/profile/discovery.rs` owns Chrome user-data-dir discovery, `Local State` profile parsing, directory/display-name/case-insensitive matching, ambiguous-name reporting, and available-profile error formatting.

## Compatibility

The split is mechanical. Owned Chrome import callers still prepare profiles through `prepare_owned_chrome_profile`, and existing `session::chrome` tests keep importing `resolve_chrome_profile` through `session::chrome::profile`.

## Validation

- `cargo test session::chrome::tests`

Standard validation for the stable point is recorded in the task status/commit that includes this decision.
