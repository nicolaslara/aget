# D385: Markdown Fixture Route Split

Decision: split markdown mock-site fixture routes into behavior-focused helpers
without changing route paths or fixture HTML.

Boundary after the split:

- `tests/mock_site_cli/aget_extractor_site/markdown.rs` keeps the `routes`
  entrypoint used by the mock-site extractor parity site.
- `markdown/basic.rs` owns basic markdown and base-link fixture routes.
- `markdown/inline.rs` owns inline/block, escaping, Unicode, and Google Docs
  fixture routes.
- `markdown/wrapping_preserve.rs` owns wrapping, reference-link paragraph,
  body-width, single-line-break, and preserved-tag fixture routes.
- `markdown/links_code_lists.rs` owns nested-list, link/image, code-whitespace,
  and ordered-list-start fixture routes.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
