# D319: Attached-Page CDP Test Split

Date: 2026-05-22

## Decision

Split `src/browser_cdp/tests/chrome/cdp_client/attached_page.rs` into smaller behavior-owned child modules while preserving the existing attached-page CDP test route.

## Boundary

- `src/browser_cdp/tests/chrome/cdp_client/attached_page.rs` now only routes attached-page CDP test modules.
- `src/browser_cdp/tests/chrome/cdp_client/attached_page/capture.rs` covers reading current URL and HTML from an existing page target without navigation.
- `src/browser_cdp/tests/chrome/cdp_client/attached_page/scan.rs` covers `scan_full_page` evaluation before final HTML capture.
- `src/browser_cdp/tests/chrome/cdp_client/attached_page/no_targets.rs` covers the no-page-target error path.

## Safety And Compatibility Notes

- This is a mechanical test decomposition only; browser/CDP runtime behavior is unchanged.
- The parent test route remains under the existing `browser_cdp::tests::chrome::cdp_client::attached_page` module.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.
- The parent test route dropped from 305 lines to 3 lines; all child modules are currently 136 lines or less.

## Validation

- `cargo fmt`
- `cargo test browser_cdp::tests::chrome::cdp_client::attached_page --lib`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
