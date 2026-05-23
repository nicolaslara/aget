# D408: Main-Content Fixture Route Split

Decision: split mock-site main-content extraction fixture routes into focused
route groups while keeping `aget_extractor_site::main_content::routes` as the
builder entrypoint used by parity coverage.

Boundary after the split:

- `main_content.rs` owns only the route composition.
- `main_content/basic.rs` owns basic main/alternative article selection,
  overlay cleanup, consent popup cleanup, and labeled-content routes.
- `main_content/density.rs` owns page-chrome rejection, link-density,
  unlabeled-density, and body-fallback routes.
- `main_content/noise_threshold.rs` owns class/id noise, embedded noise label,
  and word-threshold routes.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
