# D390: Cleanup Assertion Split

Decision: split the owned extractor cleanup parity assertion helper into
behavior-focused modules without changing the mock-site extractor coverage
entrypoint.

Boundary after the split:

- `tests/mock_site_cli/aget_extractor/cleanup.rs` keeps the
  `assert_cleanup_outputs` route used by mock-site extractor parity coverage.
- `cleanup/defaults.rs` owns default cleaned-HTML preservation and removal
  assertions.
- `cleanup/attributes.rs` owns data-attribute, explicit-attribute, and
  prettified-HTML assertions.
- `cleanup/images.rs` owns image cleanup options.
- `cleanup/links.rs` owns external, internal, social, and domain cleanup
  options.
- `cleanup/selection.rs` owns selection after attribute pruning.
- `cleanup/markdown.rs` owns markdown cleanup output assertions.
- `cleanup/support.rs` owns the shared `/html-cleanup` extraction helper.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
