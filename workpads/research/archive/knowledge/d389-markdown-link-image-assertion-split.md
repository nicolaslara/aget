# D389: Markdown Link/Image Assertion Split

Decision: split the markdown link/image parity assertion helper into
behavior-focused modules without changing the mock-site markdown coverage
entrypoint.

Boundary after the split:

- `tests/mock_site_cli/aget_extractor/markdown/links_images.rs` keeps the
  `assert_links_and_images` route used by markdown parity coverage.
- `links_images/defaults.rs` owns the default link/image markdown output
  assertion.
- `links_images/images.rs` owns image-option assertions.
- `links_images/links.rs` owns automatic-link and link-option assertions.
- `links_images/references.rs` owns inline/reference-style link assertions.
- `links_images/support.rs` owns shared escaped fixture URLs.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
