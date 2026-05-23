# D376: Owned Page Module Split

Decision: split the oversized owned page extraction module into focused page
submodules without changing extraction behavior.

Boundary after the split:

- `src/extraction/owned/page/mod.rs` keeps static-versus-rendered
  orchestration, option validation, auto-render retry selection, and direct
  rendered-HTML entrypoints.
- `src/extraction/owned/page/html.rs` owns cleaned HTML extraction, metadata
  capture, selector/exclusion sequencing, main-content preference selection,
  output-format shaping, and `prettiify` formatting.
- `src/extraction/owned/page/local_input.rs` owns `raw:`/`raw://`/`file://`
  browser-routing decisions and local temporary render input creation.
- `src/extraction/owned/page/readiness.rs` and `rendered.rs` keep their existing
  script-readiness and CDP-rendered extraction responsibilities.

The split is mechanical. It preserves owned extractor behavior and internal
call paths while reducing the former `src/extraction/owned/page.rs` load cost
for future option/readiness work.

`workpads/research/tasks.md` remains the full executable backlog and was not
compacted.
