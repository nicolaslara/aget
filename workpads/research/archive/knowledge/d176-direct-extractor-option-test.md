# Knowledge Archive D176: Direct Extractor Option Test

### D176: Broaden direct `AgetExtractor` coverage

The next I19j test slice broadened direct engine coverage in `src/aget_extractor.rs`. The unit test module now has a small local one-request HTTP fixture and covers `AgetExtractor` behavior without the `Aget` facade for:

- static markdown extraction;
- CSS selector extraction;
- `exclude_selector` cleanup;
- `crawl4ai.target_elements` option handling.

The new test intentionally avoids requiring Chrome and keeps the stable public extractor metadata unchanged.

Validation:

- `cargo fmt --check`
- `cargo test aget_extractor::tests::`
- `cargo test`
- `git diff --check`
- `rg -n "OwnedExtractorBackend|OwnedBrowserAutomationBackend|DefaultExtractorBackend::Owned|DefaultBrowserAutomationBackend::Owned" src tests README.md .opencode .cursor`

Confidence: High for the test coverage addition. It is local, deterministic, and directly exercises the engine boundary I19j is introducing.
