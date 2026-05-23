# D415: Markdown Block Renderer Split

Decision: split markdown block rendering helpers into focused modules while
preserving the function names and visibility used by markdown dispatch.

Boundary after the split:

- `src/extraction/markdown/block/mod.rs` routes block-rendering helpers.
- `src/extraction/markdown/block/structure.rs` owns headings, generic blocks,
  horizontal rules, and structural block detection.
- `src/extraction/markdown/block/lists.rs` owns ordered/unordered lists,
  Google Docs list indentation, and definition lists.
- `src/extraction/markdown/block/code.rs` owns fenced pre/code rendering and
  `handle_code_in_pre` marker preservation.
- `src/extraction/markdown/block/quote.rs` owns blockquote rendering.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
