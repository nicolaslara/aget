# Knowledge Archive D180: Extraction Helper Module Split

### D180: Split output and replay-scope helpers from extraction orchestration

The first I19o mechanical split kept `src/extraction/mod.rs` as the `aget get` orchestration module and moved two cohesive helper groups out:

- `src/extraction/output.rs`: `OutputOptions`, output option capture, and content limit metadata application.
- `src/extraction/replay_scope.rs`: session replay-scope enforcement, host/domain matching, and replay-scope unit tests.

The extraction module interface remains unchanged for callers. `OutputOptions` is re-exported from `extraction`, and `domain_matches_host` remains available inside the extraction module for HTTP cookie filtering.

Validation:

- `cargo fmt`
- `cargo test extraction::`
- `cargo test --test get_cli session::get_rejects_session_replay_outside_saved_scope`
- `cargo test --test mock_site_cli backend_parity_covers_extractor_content_formats`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`

Confidence: High for the mechanical split. The moved replay-scope tests still cover subdomain matching and mixed-domain rejection, and the full suite passed before marking I19o complete.
