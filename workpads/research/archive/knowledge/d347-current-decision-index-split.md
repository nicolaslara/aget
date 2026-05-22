# D347: Current Decision Index Split

## Decision

Keep `workpads/research/archive/knowledge/current-decision-index.md` as a compact router and move dense historical per-decision rows into smaller child index files.

## Context

The user clarified that `workpads/research/tasks.md` must not be compacted because it preserves planned and missing task context. The support-file compaction target is dense reference material, not the executable task list.

## Implementation

- Added `workpads/research/archive/knowledge/decision-index/d168-d231.md`.
- Added `workpads/research/archive/knowledge/decision-index/d232-d292.md`.
- Added `workpads/research/archive/knowledge/decision-index/d293-d346.md`.
- Rewrote `current-decision-index.md` as a compact router with recent decisions, full archive routing, and current open migration gaps.
- Preserved `workpads/research/tasks.md` as the planned-task source of truth.

## Validation

- Markdown routing audit: child indexes exist, referenced recent decisions exist, and `tasks.md` remains un-compacted.

## Follow-Up

- Future support-file compaction should keep dense completed-history details in child files and keep top-level support files as routers.
