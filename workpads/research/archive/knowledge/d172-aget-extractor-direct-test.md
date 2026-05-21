# Knowledge Archive D172: AgetExtractor Direct Test And Naming Cleanup

### D172: Exercise `AgetExtractor` directly and rename active extractor tests

The next I19j slice added a crate-local `AgetExtractor` unit test that serves a tiny local HTML page and calls the engine directly with `ExtractorRequest`, bypassing the public `Aget` facade and CLI envelope. This gives the new engine boundary direct coverage instead of only proving adapter wiring through `AgetExtractorBackend`.

Active mock-site extractor test files and function names now use `aget_extractor` naming instead of the migration-era `owned` label. Current README and OpenCode tool descriptions also refer to `AgetExtractor` for supported extraction backend options. The public extractor identifier still remains `aget-owned-extractor` in this slice to avoid changing existing envelope behavior while the refactor is mechanical.

Validation:

- `cargo test aget_extractor::tests::extracts_static_page_without_aget_facade`
- `cargo test --test mock_site_cli`
- `cargo test --test mock_site_browser`
- `cargo fmt --check`

Confidence: High. This is a naming and test-boundary slice with no intended extraction behavior change.
