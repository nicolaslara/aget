# D393: Aget Backend Facade Split

Decision: split the Aget facade backend helpers into behavior-focused modules
without changing public facade exports or backend behavior.

Boundary after the split:

- `src/aget/backends/mod.rs` keeps the compact public re-export surface used by
  `src/aget/mod.rs` and external callers.
- `backends/extractor.rs` owns default extractor backend selection between the
  owned `AgetExtractorBackend` and compatibility command adapter.
- `backends/browser.rs` owns the browser automation capability trait and default
  browser backend selection/delegation.
- `backends/current_tab.rs` owns current-tab request/result types and the
  current-tab backend trait.
- `backends/owned_browser.rs` owns the facade adapter from `AgetBrowser` into
  browser automation, fallback extraction, and current-tab capabilities.
- `backends/command_browser.rs` owns the compatibility command browser
  automation/fallback adapter and its explicit current-tab unavailable error.

The public paths re-exported from `aget::aget` remain stable:
`AgetBrowserBackend`, `BrowserAutomationBackend`, `BrowserCurrentTabBackend`,
`BrowserCurrentTabRequest`, `CommandBrowserAutomationBackend`,
`DefaultBrowserAutomationBackend`, and `DefaultExtractorBackend`.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
