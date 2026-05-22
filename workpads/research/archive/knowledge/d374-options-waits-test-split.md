# D374: AgetExtractor Options/Waits Test Split

Date: 2026-05-23

## Decision

The oversized AgetExtractor options/waits parity helper was split into behavior-focused child modules while preserving the existing integration-test route and assertions.

`workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Implementation Notes

- `tests/mock_site_cli/aget_extractor/options_waits.rs` is now a small router.
- Positive option behavior lives in `tests/mock_site_cli/aget_extractor/options_waits/content.rs`.
- Backend option validation and unsupported-option diagnostics live in `tests/mock_site_cli/aget_extractor/options_waits/validation.rs`.
- Redirect and wait-selector behavior lives in `tests/mock_site_cli/aget_extractor/options_waits/waits.rs`.
- No production code or fixture behavior changed.

## Validation

Passed:

- `cargo test aget_extractor_backend_covers_static_http_parity_slice`
- Line-count check: router 16 lines, child modules 107/316/32 lines, compact support routers unchanged.
- `cargo fmt --check`
- `git diff --check`
- `cargo test`
