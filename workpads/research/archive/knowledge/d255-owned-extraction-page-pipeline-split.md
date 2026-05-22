# D255: Owned Extraction Page Pipeline Split

## Decision

Split owned page extraction/routing helpers out of `src/extraction/owned/mod.rs`.

## Boundary

- `src/extraction/owned/mod.rs` keeps backend adapter entrypoints, compatibility labels, artifact writes, and public owned extractor exports.
- `src/extraction/owned/page.rs` owns static-versus-rendered routing, rendered-page request construction, wait-selector retry, direct rendered-HTML extraction, script detection, HTML cleanup sequencing, selector/fallback selection, and output-format shaping.
- `src/extraction/owned/content.rs`, `options.rs`, and `text.rs` remain the content selection, option parsing, and text rendering submodules.

## Compatibility

The split is mechanical. `run_owned_extractor_backend`, `run_owned_browser_fallback`, `extract_owned_rendered_html`, `validate_owned_extraction_options`, and `OWNED_EXTRACTOR` stay available to existing callers. Static fetch, rendered fallback, script detection, cleanup, selectors, warnings, artifacts, and output formats are unchanged.

## Validation

- `cargo test extraction::owned`
- `cargo test --test mock_site_cli aget_extractor`

Standard validation for the stable point is recorded in the task status/commit that includes this decision.
