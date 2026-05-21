# D193: Extraction Types and Backend Adapter Split

The extraction parent module was mechanically reduced while preserving public extraction API paths through `pub use` re-exports from `src/extraction/mod.rs`.

Resulting boundaries:

- `src/extraction/mod.rs`: extraction orchestration, session loading/composition, replay-scope enforcement, run artifact finalization, fallback selection, and shared error helpers.
- `src/extraction/types.rs`: public `GetOptions`, `GetSuccess`, artifact/timing/limit DTOs, extractor/fallback request/result DTOs, backend traits, and `ExtractionSessionStore`.
- `src/extraction/backends.rs`: `AgetExtractorBackend`, `CommandExtractorBackend`, and `CommandBrowserFallbackBackend` adapter structs.

The split did not intentionally change default backend selection, public re-export paths, extractor/fallback trait contracts, artifact writing, replay checks, or error mapping.

Validation:

- `cargo fmt`
- `cargo fmt --check`
- `cargo test extraction::`
- `cargo test --test aget_api`
- `cargo test --test mock_site_cli aget_extractor`
- `cargo test` after rerunning transient `session_cli::authorize` localhost failures successfully
- `git diff --check`
