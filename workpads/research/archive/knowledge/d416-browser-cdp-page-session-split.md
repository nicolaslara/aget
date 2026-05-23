# D416: Browser CDP Page Session Split

Decision: split owned CDP page-session helpers into focused modules while
preserving the `PageSession` type and `CdpClient` method names used by render,
login, state export/import, and tests.

Boundary after the split:

- `src/browser_cdp/client/page/mod.rs` routes page-session helpers and keeps
  `PageSession` plus session-parameter routing.
- `src/browser_cdp/client/page/targets.rs` owns page creation, existing-page
  attach, direct-page WebSocket sessions, flattened target attachment, and
  stable missing-field errors.
- `src/browser_cdp/client/page/domains.rs` owns page/runtime/network domain
  enabling and auto-attach setup.
- `src/browser_cdp/client/page/preload.rs` owns shadow-root preload injection.
- `src/browser_cdp/client/page/emulation.rs` owns user-agent, locale, and
  timezone overrides.
- `src/browser_cdp/client/page/close.rs` owns browser and page close helpers.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
