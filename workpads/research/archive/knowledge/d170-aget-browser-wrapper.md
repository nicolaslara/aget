# Knowledge Archive D170: AgetBrowser Wrapper

### D170: Route default browser automation through `AgetBrowserBackend`

The first I19j code slice introduced `src/aget_browser.rs` with `AgetBrowser` as the local browser/CDP engine boundary. The new `AgetBrowserBackend` wraps that engine and now backs `DefaultBrowserAutomationBackend::Owned`, so the default browser automation path starts moving away from direct migration-era `OwnedBrowserAutomationBackend` wiring.

This slice is intentionally mechanical. `AgetBrowser` still delegates to the existing owned Chrome import, login lifecycle, and browser-fallback slices. `OwnedBrowserAutomationBackend` remains as a compatibility shim for existing tests/callers and delegates to `AgetBrowserBackend`; later I19j slices can rename remaining live call sites once the extractor/browser engine boundaries are stable.

Validation:

- `cargo check`
- `cargo test --test aget_api`
- `cargo test --test mock_site_browser`
- `cargo fmt --check`
- `git diff --check`

Confidence: High. The change adds a wrapper boundary and direct backend wiring without intended behavior changes; focused API and mock-site browser tests pass.
