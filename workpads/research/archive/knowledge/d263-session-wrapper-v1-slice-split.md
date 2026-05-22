# D263: Split Historical V1 Thin-Wrapper Support Slice

## Decision

Split the dense historical V1 implementation spec support file into smaller routed archive files without compacting or removing planned tasks.

## Boundary

- `workpads/research/archive/support/session-wrapper-poc-spec/v1-implementation-spec.md` is now a compact routing index.
- `v1-architecture-storage.md` owns the historical thin-wrapper architecture, dependency, storage layout, and session file shape.
- `v1-command-surface.md` owns the historical CLI command surface for `aget get` and session commands.
- `v1-adapters-composition-output.md` owns the historical Crawl4AI/cmux/agent-browser adapter notes, Playwright state composition, and run metadata shape.

## Rationale

The active task backlog must remain complete and un-compacted, but supporting archive files should stay small enough for agents to load only the context they need. This preserves historical evidence while keeping the old V1 implementation entrypoint as a low-token router.

## Validation

- `rg -n "v1-(architecture-storage|command-surface|adapters-composition-output)|V1 Implementation Spec" workpads/research/archive/support/session-wrapper-poc-spec.md workpads/research/archive/support/session-wrapper-poc-spec`
- `git ls-files 'workpads/research/*' | xargs wc -l | sort -nr | sed -n '1,40p'`
- `git diff --check`
