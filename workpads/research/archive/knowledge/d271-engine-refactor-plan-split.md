# D271: Aget Engine Refactor Plan Split

## Decision

Keep `workpads/research/aget-engine-refactor-plan.md` as the stable routing entrypoint, but move the dense historical engine-refactor plan detail into smaller section files:

- `aget-engine-refactor-plan/goal-boundaries-naming.md`
- `aget-engine-refactor-plan/migration-testing-guardrails.md`

This preserves `workpads/research/tasks.md` as the full planned-task source of truth. The task file was not compacted; it only records the completed support-file split.

## Boundary

The split is documentation-only. It preserves the existing plan path for tasks that still tell agents to read it before engine-boundary code changes.

## Validation

- `wc -l workpads/research/aget-engine-refactor-plan.md workpads/research/aget-engine-refactor-plan/*.md workpads/research/archive/knowledge/d271-engine-refactor-plan-split.md workpads/research/tasks.md`
- `git diff --check`
