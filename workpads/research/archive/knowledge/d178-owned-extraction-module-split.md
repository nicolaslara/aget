# Knowledge Archive D178: Owned Extraction Module Split

### D178: Split owned extraction orchestration internals

The first I19m mechanical split moved `src/extraction/owned.rs` to a module directory:

- `src/extraction/owned/mod.rs`: backend entrypoints, static-vs-rendered orchestration, rendered-page handoff, HTML pre-cleanup, and output-format selection.
- `src/extraction/owned/options.rs`: supported `crawl4ai.*` backend option parsing and validation, including CSS-only waits.
- `src/extraction/owned/content.rs`: selector fallback, target-element selection, main-content scoring, cleaned HTML serialization, markdown rendering, and text normalization.

The extraction module interface remains unchanged for callers: `run_owned_extractor_backend`, `run_owned_browser_fallback`, and `OWNED_EXTRACTOR` stay exported through `extraction::owned`.

Validation:

- `cargo fmt`
- `cargo test aget_extractor::tests::`
- `cargo test --test mock_site_cli aget_extractor::aget_extractor_backend_covers_static_http_parity_slice`
- `cargo test --test mock_site_cli backend_parity_covers_extractor_content_formats`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

Confidence: High. This slice is mechanical, focused owned-extractor coverage passed, and the full suite passed.
