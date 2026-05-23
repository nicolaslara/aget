# D386: Get Validation Test Split

Decision: split get CLI validation tests into behavior-focused helpers without
changing validation coverage.

Boundary after the split:

- `tests/get_cli/validation.rs` keeps the `validation` module entrypoint used
  by `tests/get_cli.rs`.
- `tests/get_cli/validation/options.rs` owns the Crawl4AI backend-option
  forwarding matrix.
- `tests/get_cli/validation/rejections.rs` owns unsupported extractor-option
  and JavaScript wait rejection coverage plus shared JSON/metadata error
  assertions.

The split preserves the existing test names and command assertions.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
