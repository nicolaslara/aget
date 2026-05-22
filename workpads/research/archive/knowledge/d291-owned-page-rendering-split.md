# D291: Owned Page Rendering Helper Split

Date: 2026-05-22

## Decision

Split rendered-page request composition and script/readiness detection out of `src/extraction/owned/page.rs` while preserving owned extraction behavior.

## Boundary

- `src/extraction/owned/page.rs` remains the owned page extraction coordinator. It keeps `OwnedPageExtraction`, `extract_owned_static_or_rendered`, direct rendered-HTML extraction, HTTP response handoff, HTML cleanup sequencing, selector/fallback selection, base-URL handling, main-content preference, output-format shaping, and page metadata propagation.
- `src/extraction/owned/page/rendered.rs` owns CDP render request composition and the rendered-wait retry predicate used when a CSS wait selector is missing from static HTML.
- `src/extraction/owned/page/readiness.rs` owns executable script detection for static-versus-rendered routing, including the existing assertions for common executable JavaScript types and non-executable JSON/import-map/speculation-rules script types.

## Safety And Compatibility Notes

- This is a mechanical decomposition only. Browser render request fields, static-versus-rendered routing, rendered wait retry behavior, script type classification, output shaping, and metadata behavior are unchanged.
- The JavaScript-wait/user-script safety boundary is unchanged; this split does not add execution of user-provided JavaScript.
- `workpads/research/tasks.md` remains the full executable backlog and was not compacted.

## Validation

- `cargo test extraction::owned::page::readiness --lib`
- `cargo test aget_extractor::aget_extractor_backend_covers_static_http_parity_slice --test mock_site_cli`
- `cargo test --test mock_site_browser`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
