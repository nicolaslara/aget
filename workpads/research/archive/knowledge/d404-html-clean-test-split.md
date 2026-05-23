# D404: HTML Clean Test Split

Decision: split owned HTML cleanup URL/link/image unit tests out of
`src/extraction/html_clean/mod.rs` while keeping the existing
`extraction::html_clean::{...}` helper paths unchanged for owned extraction
callers.

Boundary after the split:

- `html_clean/mod.rs` owns the cleanup route, selector parsing, overlay/consent
  selector constants, and helper re-export surface.
- `html_clean/tests.rs` owns the focused URL, link, image, excluded-domain, and
  social-media cleanup assertions.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
