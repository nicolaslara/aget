# D197: Support-file compaction without task compaction

## Decision

Top-level support workpad files should stay small enough for agents to load at task switches. Dense historical material belongs under `workpads/research/archive/` and should be reached through explicit routing links.

`workpads/research/tasks.md` remains the detailed executable backlog. This pass intentionally did not compact completed or planned task entries.

## Changes

- Replaced `workpads/research/session-wrapper-poc-spec.md` with a routing stub.
- Moved the full historical PoC spec to `workpads/research/archive/support/session-wrapper-poc-spec.md`.
- Replaced `workpads/research/session-wrapper-implementation-plan.md` with a routing stub.
- Moved the full historical implementation plan to `workpads/research/archive/support/session-wrapper-implementation-plan.md`.
- Moved dense top-level `knowledge.md` decision routing into `workpads/research/archive/knowledge/current-decision-index.md`.

## Validation

- Confirm archived files exist and old top-level paths still route to them.
- Confirm `tasks.md` remains the detailed backlog rather than a compact summary.
- Run Markdown link/path checks with `test -f` and diff hygiene with `git diff --check`.
