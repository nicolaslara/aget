# D380: Markdown Writer Helper Split

Decision: split markdown writer state helpers into focused submodules without
changing markdown output behavior.

Boundary after the split:

- `src/extraction/markdown/writer/mod.rs` keeps `MarkdownWriter` state,
  constructor/child cloning, URL resolution, tag-preservation checks, and image
  alt fallback.
- `src/extraction/markdown/writer/text.rs` owns text normalization before inline
  output, spacing-sensitive inline pushes, and blank-line normalization.
- `src/extraction/markdown/writer/references.rs` owns abbreviation collection,
  reference-style link numbering, paragraph-scoped reference flushing, final
  reference definitions, and abbreviation definition output.

The split preserves the existing `writer::MarkdownWriter` path used by
`src/extraction/markdown/mod.rs`, `block.rs`, `inline.rs`, and `table/`.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
