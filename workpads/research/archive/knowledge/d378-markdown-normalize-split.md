# D378: Markdown Normalize Helper Split

Decision: split the oversized markdown normalization helper module into focused
submodules without changing rendering behavior.

Boundary after the split:

- `src/extraction/markdown/normalize/mod.rs` keeps whole-document markdown
  normalization, inline whitespace normalization, Unicode-snob replacements,
  punctuation spacing checks, and trailing whitespace/newline helpers.
- `src/extraction/markdown/normalize/wrap.rs` owns body-width wrapping,
  single-line-break application, and line-preservation rules for code fences,
  headings, tables, lists, links, and blockquotes.
- `src/extraction/markdown/normalize/escape.rs` owns Markdown text escaping,
  line-start marker escaping, link/text/title escaping, and table-cell escaping.
- `src/extraction/markdown/normalize/url.rs` owns absolute HTTP checks and
  base-URL-aware markdown target resolution.

The split preserves the existing `super::normalize::*` imports used by the
markdown renderer modules.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
