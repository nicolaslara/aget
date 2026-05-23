# D412: Markdown Dispatch Google Docs Split

Decision: split Google Docs-specific Markdown dispatch helpers out of the
generic tag traversal and dispatch route while preserving existing
`render_node` and `render_children` call paths.

Boundary after the split:

- `src/extraction/markdown/dispatch/mod.rs` remains the traversal and generic
  tag-dispatch entrypoint.
- `src/extraction/markdown/dispatch/google_doc.rs` owns Google Docs styled
  inline rendering and `list-style-type` ordered/unordered list detection.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
