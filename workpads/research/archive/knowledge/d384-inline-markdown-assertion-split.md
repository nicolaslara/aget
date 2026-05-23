# D384: Inline Markdown Assertion Split

Decision: split inline markdown parity assertions into behavior-focused helper
modules without changing mock-site extractor coverage.

Boundary after the split:

- `tests/mock_site_cli/aget_extractor/markdown/inline_blocks.rs` keeps the
  `assert_inline_blocks` route used by `assert_markdown_rendering`.
- `inline_blocks/semantics.rs` owns semantic inline/block assertions for
  emphasis, quotes, strikethrough, figures, details, and definitions.
- `inline_blocks/escape_unicode.rs` owns line-start escaping, backslash
  escaping, escape-snob, and Unicode-snob assertions.
- `inline_blocks/google_preserve.rs` owns Google Docs style and raw preserved
  tag assertions.
- `inline_blocks/wrapping.rs` owns body-width, list/table/link wrapping, and
  single-line-break assertions.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
