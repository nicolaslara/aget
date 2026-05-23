# D381: Markdown Render Dispatch Split

Decision: split markdown node traversal and tag dispatch out of the renderer
entrypoint without changing markdown output behavior.

Boundary after the split:

- `src/extraction/markdown/mod.rs` keeps the `element_to_markdown` entrypoint,
  `MarkdownWriter` setup, final reference/abbreviation flushing, normalization,
  wrapping, and table padding.
- `src/extraction/markdown/dispatch.rs` owns DOM node traversal, tag dispatch,
  preserved raw HTML output, Google Docs inline/list style handling,
  line-through hiding, and `only_text` inline-tag handling.

The split preserves the existing `element_to_markdown` caller path and the
internal `render_node`/`render_children` routes used by markdown submodules.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
