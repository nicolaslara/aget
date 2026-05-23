# D383: Owned Content Extraction Split

Decision: split owned content extraction helpers into behavior-focused modules
without changing selection, cleanup, rendering, or markdown base-URL behavior.

Boundary after the split:

- `src/extraction/owned/content/mod.rs` keeps the `extract_owned_content`
  orchestration, cleanup ordering, and `markdown_base_url` route.
- `src/extraction/owned/content/selection.rs` owns selected-root detection,
  target-element collection, and selected-element lookup.
- `src/extraction/owned/content/cleanup.rs` owns owned line-through element
  removal.
- `src/extraction/owned/content/render.rs` owns single-element and multi-target
  HTML/markdown/text output construction.
- `src/extraction/owned/content/main_content.rs` remains the default
  main-content scoring module.

The split preserves the existing `content::extract_owned_content` and
`content::markdown_base_url` call paths used by
`src/extraction/owned/page/html.rs`.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
