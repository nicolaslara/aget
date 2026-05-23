# D413: Owned HTTP Helper Split

Decision: split owned static HTTP extraction helpers into focused modules while
preserving the `owned_fetch` entrypoint used by the owned extractor.

Boundary after the split:

- `src/extraction/http/mod.rs` keeps the `owned_fetch` route, ureq request
  execution, user-agent forwarding, and ureq error mapping.
- `src/extraction/http/source.rs` owns URL/source parsing plus local
  `file://`, `raw:`, and `raw://` response creation.
- `src/extraction/http/cookies.rs` owns session-cookie replay filtering and
  request cookie-header construction.
- `src/extraction/http/tests.rs` owns the local-input unit coverage.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
