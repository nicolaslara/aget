# D288: Browser CDP Discovery Test Split

Decision: browser CDP discovery tests now route through smaller behavior-focused test modules.

Implementation boundary:

- `src/browser_cdp/tests/discovery.rs` is now a module index.
- `src/browser_cdp/tests/discovery/diagnostics.rs` owns Chrome stderr URL parsing, startup classification, sandbox hints, and generic stderr-tail coverage.
- `src/browser_cdp/tests/discovery/port_file.rs` owns `DevToolsActivePort` parsing, early-exit/stderr fallback coverage, and stale port-file cleanup assertions.
- `src/browser_cdp/tests/discovery/endpoint.rs` owns `/json/version`, `/json/list`, host-rewrite, and direct-WebSocket CDP discovery fallback coverage.
- Production discovery behavior, test assertions, helper semantics, and runtime code are unchanged.
- `workpads/research/tasks.md` was not compacted.

Validation:

- `cargo test browser_cdp::tests::discovery --lib`
- `cargo fmt --check`
- `cargo test`
- `git diff --check`
