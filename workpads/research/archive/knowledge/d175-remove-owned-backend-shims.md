# Knowledge Archive D175: Remove Owned Backend Shims

### D175: Remove live `Owned*Backend` compatibility shim types

The next I19j mechanical slice removed the active compatibility shim types `OwnedExtractorBackend` and `OwnedBrowserAutomationBackend`. Their live call sites had already moved to `AgetExtractorBackend` and `AgetBrowserBackend`, so keeping the old public names only preserved migration-era vocabulary.

The default backend enums now use `Aget` variants for the local engine path:

- `DefaultExtractorBackend::Aget(AgetExtractorBackend)`
- `DefaultBrowserAutomationBackend::Aget(AgetBrowserBackend)`

This does not change the stable CLI/API output strings. Existing extraction metadata such as `aget-owned-extractor` and `aget-owned-browser-fallback` remains unchanged in this slice to avoid envelope churn while the refactor is mechanical.

Validation:

- `cargo fmt --check`
- `cargo test aget_extractor::tests::extracts_static_page_without_aget_facade`
- `cargo test aget_browser::tests::cancels_pending_login_without_aget_facade_or_command_backend`
- `cargo test`

Confidence: Medium-high. The slice is a narrow live-code rename/removal with focused engine tests; full-suite validation is still required before considering I19j complete.
