# D396: Markdown Inline Renderer Split

Decision: split the owned markdown inline renderer into behavior-focused
modules without changing inline rendering behavior or call paths.

Boundary after the split:

- `src/extraction/markdown/inline/mod.rs` keeps the compact route and re-export
  surface used by markdown dispatch and block rendering.
- `inline/link.rs` owns anchor rendering, automatic-link detection, linked
  image handling, heading-link rendering, reference-link labels, and protected
  link targets.
- `inline/image.rs` owns image markdown/HTML/alt/reference rendering.
- `inline/text.rs` owns abbreviation rendering, child inline markdown
  collection, raw text collection, and inline text normalization.

The existing markdown dispatch/block imports remain stable:
`inline_markdown_from_children`, `inline_text_from_node`, `raw_text_from_node`,
`render_abbreviation`, `render_image`, and `render_link`.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
