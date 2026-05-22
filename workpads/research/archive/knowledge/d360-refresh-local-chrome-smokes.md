# D360: Refresh Local Chrome Smoke Expectations

## Decision

Ignored local Chrome smokes now assert the same owned text block-boundary contract as deterministic text extraction tests.

## Source Evidence

- `references/repos/agent-browser`, commit `3bb1d43f8bb16444596365496f78395da8f1e6b7`, `cli/src/native/browser.rs`.
- agent-browser enables Page/Runtime/Network domains, resumes waiting-for-debugger targets, enables auto-attach for subtargets, and supports local Chrome/CDP paths used by the owned browser parity tests.
- Current owned extractor text output preserves block boundaries with newline-separated headings/paragraph blocks.

## Implementation

- Updated stale ignored Chrome smoke assertions for image readiness, networkidle readiness, shadow DOM flattening, and render-delay behavior from inline text to newline-separated text.
- No extractor or browser behavior changed.
- `workpads/research/tasks.md` remains the full task backlog and was not compacted.

## Validation

- `cargo test owned_chrome_import_exports_cookie_and_local_storage_from_profile_directory --lib -- --ignored`
- Initial `cargo test --test mock_site_browser -- --ignored` found four stale newline expectations.
- `cargo test --test mock_site_browser -- --ignored`
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `cargo test`

## Follow-Up

- I19e remains open for manual headed login/profile/keychain smoke and final review evidence.
- I19h remains open until review subagents are explicitly allowed/requested.
