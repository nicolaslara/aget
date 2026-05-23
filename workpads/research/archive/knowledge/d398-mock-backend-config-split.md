# D398: Mock Backend Config Split

Decision: split the `aget_mock_backend` config helper into behavior-focused
modules without changing the mock backend binary's helper call paths or test
support behavior.

Boundary after the split:

- `tests/support/bin/aget_mock_backend/config/mod.rs` owns config-file loading
  and re-exports the helper surface used by the mock backend binary.
- `config/expectations.rs` owns expected argument and environment assertions.
- `config/options.rs` owns Crawl4AI option validation plus structured
  validation error metadata/output.
- `config/state.rs` owns Playwright-state assertions, state-file loading, and
  state placeholder expansion.

The mock backend binary still calls `config::read_config`,
`config::assert_expected_args`, `config::assert_expected_environment`,
`config::assert_expected_state`, `config::read_state`, and
`config::expand_state_placeholders`.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
