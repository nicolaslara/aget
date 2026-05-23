# D382: Owned Option Apply Split

Decision: split owned extractor backend-option application into behavior-focused
helpers without changing parsing or unsupported-option diagnostics.

Boundary after the split:

- `src/extraction/owned/options/apply/mod.rs` keeps the public
  `apply_owned_extractor_option` route and the canonical supported-option list.
- `src/extraction/owned/options/apply/document.rs` owns document selection,
  cleanup, metadata/request-identity, and cleaned-HTML option application.
- `src/extraction/owned/options/apply/markdown.rs` owns markdown, link, image,
  table, typography, wrapping, and social-link option application.
- `src/extraction/owned/options/apply/browser.rs` owns browser/readiness,
  timeout, scrolling, iframe, shadow-DOM, and word-threshold option application.

The split preserves the existing `apply_owned_extractor_option` call path used
by `src/extraction/owned/options.rs`.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
