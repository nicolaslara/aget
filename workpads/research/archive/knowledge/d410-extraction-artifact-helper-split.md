# D410: Extraction Artifact Helper Split

Decision: split artifact-related extraction helpers into focused modules while
preserving the existing `src/extraction` import surface for run metadata,
private artifact files, session-backed redaction, and backend error
sanitization.

Boundary after the split:

- `src/extraction/artifacts/mod.rs` is the compact route and run-id helper.
- `src/extraction/artifacts/redaction.rs` owns sensitive session value
  collection, backend error sanitization, stdout/stderr artifact redaction, and
  percent/form/JSON-escaped redaction patterns.
- `src/extraction/artifacts/metadata.rs` owns success and error metadata JSON
  serialization.
- `src/extraction/artifacts/private_files.rs` owns private run directory and
  file creation plus output-file reads.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
