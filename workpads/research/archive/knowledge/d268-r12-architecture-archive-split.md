# D268: R12 Architecture Proposal Archive Split

## Decision

Keep `workpads/research/archive/knowledge/r12-mvp-architecture-proposal.md` as the stable routing entrypoint, but move the dense historical R12 proposal detail into smaller section files:

- `r12-mvp-architecture-proposal/architecture-boundary-cli.md`
- `r12-mvp-architecture-proposal/config-session-output.md`
- `r12-mvp-architecture-proposal/privacy-integration-implementation.md`

This preserves `workpads/research/tasks.md` as the full planned-task source of truth. The task file was not compacted; it only records the completed support-file split.

## Boundary

The split is documentation-only. It does not change product direction, active migration scope, runtime behavior, or verification expectations.

## Validation

- `git ls-files 'workpads/**' 'project.md' 'WORKING.md' | xargs wc -l | sort -nr | head -40`
- `git diff --check`
