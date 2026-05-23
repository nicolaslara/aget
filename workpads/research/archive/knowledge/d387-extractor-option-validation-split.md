# D387: Extractor Option Validation Split

Decision: split AgetExtractor backend-option validation assertions into
behavior-focused helpers without changing validation coverage.

Boundary after the split:

- `tests/mock_site_cli/aget_extractor/options_waits/validation.rs` keeps the
  `assert_option_validation` route used by mock-site extractor parity coverage.
- `validation/document.rs` owns document/cache/base-URL and unsupported-option
  diagnostics.
- `validation/markdown.rs` owns markdown, link, table, typography, wrapping,
  and Google Docs validation diagnostics.
- `validation/browser.rs` owns browser/readiness, timeout, iframe, and scrolling
  validation diagnostics.
- `validation/support.rs` owns the shared invalid-option assertion helper.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
