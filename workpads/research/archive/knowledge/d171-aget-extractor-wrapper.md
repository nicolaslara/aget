# Knowledge Archive D171: AgetExtractor Wrapper

### D171: Route default extraction through `AgetExtractorBackend`

The second I19j code slice introduced `src/aget_extractor.rs` with `AgetExtractor` as the local Crawl4AI-like extraction engine boundary. `AgetExtractorBackend` now wraps that engine and backs `DefaultExtractorBackend::Owned`, and the standalone `get_url` helpers now use `AgetExtractorBackend` plus `AgetBrowserBackend` by default.

This slice is intentionally mechanical. `AgetExtractor` still delegates to the existing owned extraction entrypoint, and `OwnedExtractorBackend` remains as a compatibility shim for existing callers/tests. Mock-site extractor and browser tests now exercise `AgetExtractorBackend` directly where they cover the active local engine path.

Validation:

- `cargo check`
- `cargo test --test mock_site_cli`
- `cargo test --test mock_site_browser`
- `cargo fmt --check`
- `git diff --check`

Confidence: High. The change adds an adapter boundary and default wiring without intended behavior changes; focused extraction and browser tests pass.
