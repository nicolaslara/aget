# D417: Extraction Pipeline Helper Split

Decision: split extraction pipeline helpers into focused modules while
preserving the function names and visibility used by `src/extraction/mod.rs`
and direct owned extractor callers.

Boundary after the split:

- `src/extraction/pipeline/mod.rs` routes pipeline helpers and owns the
  `SuccessfulExtraction` transfer type.
- `src/extraction/pipeline/direct.rs` owns direct extraction finishing for
  caller-provided content.
- `src/extraction/pipeline/backend.rs` owns primary extractor execution and
  session-backed browser fallback conversion.
- `src/extraction/pipeline/finalization.rs` owns success/error finalization,
  private content writing, metadata writing, output limits, and sensitivity
  handling.
- `src/extraction/pipeline/sessions.rs` owns selected-session loading.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
