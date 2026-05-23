# D414: Mock-Site Browser Rendering Test Split

Decision: split the local Chrome rendering mock-site tests into focused modules
without changing the ignored smoke coverage, test names, assertions, or module
target.

Boundary after the split:

- `tests/mock_site_browser/rendering/mod.rs` routes rendering scenario modules.
- `tests/mock_site_browser/rendering/basic.rs` owns waited JavaScript and
  scripted-page rendering coverage.
- `tests/mock_site_browser/rendering/local_input.rs` owns raw content processed
  in browser.
- `tests/mock_site_browser/rendering/shadow_iframe.rs` owns shadow DOM
  flattening and accessible iframe processing.
- `tests/mock_site_browser/rendering/delay.rs` owns render-delay coverage.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
