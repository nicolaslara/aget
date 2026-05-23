# D379: Markdown Table Renderer Split

Decision: split the oversized markdown table renderer module into focused
submodules without changing markdown output behavior.

Boundary after the split:

- `src/extraction/markdown/table/mod.rs` keeps the public table-rendering
  orchestration and header/body/caption output sequencing.
- `src/extraction/markdown/table/rows.rs` owns caption discovery, table row
  collection, cell extraction, cell escaping, row normalization, and GFM row
  formatting.
- `src/extraction/markdown/table/ignored.rs` owns `crawl4ai.ignore_tables`
  plain-text table extraction.
- `src/extraction/markdown/table/bypass.rs` owns `crawl4ai.bypass_tables`
  HTML-like table rendering.
- `src/extraction/markdown/table/pad.rs` owns `crawl4ai.pad_tables` alignment
  formatting after markdown generation.

The split preserves existing `table::render_table` and
`table::pad_markdown_tables` call paths used by the markdown renderer.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
