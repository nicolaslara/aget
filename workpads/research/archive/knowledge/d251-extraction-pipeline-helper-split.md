# D251: Extraction Pipeline Helper Split

## Decision

Split extraction pipeline helper code out of `src/extraction/mod.rs` into `src/extraction/pipeline.rs`.

## Boundary

- `src/extraction/mod.rs` keeps the public extraction module facade, `get_url`/`get_url_with_*` orchestration entrypoints, artifact helper routing, replay-scope enforcement, and compatibility exports.
- `src/extraction/pipeline.rs` owns selected-session loading, primary extractor execution, authenticated-session fallback, direct extraction finalization, and success/error envelope finalization.
- `finish_direct_extraction` remains available as `crate::extraction::finish_direct_extraction` for internal current-tab callers.

## Compatibility

The split is mechanical. It preserves public `crate::extraction::*` paths, `get_url`/`get_url_with_*` behavior, run artifact paths, metadata writing, sensitive fallback behavior, output limits, and direct current-tab extraction output shaping.

## Validation

- `cargo test extraction`
- `cargo test current_tab`

Standard validation for the stable point is recorded in the task status/commit that includes this decision.
