# D264: Split Historical Session-Wrapper Implementation Plan

## Decision

Split the dense historical session-wrapper implementation plan support file into smaller routed archive files without compacting or removing planned tasks.

## Boundary

- `workpads/research/archive/support/session-wrapper-implementation-plan.md` is now a compact routing index.
- `session-wrapper-implementation-plan/objective-stack-repo.md` owns the historical objective, implementation principle, stack, and repository shape.
- `session-wrapper-implementation-plan/milestones.md` owns milestones 0-7 and their acceptance criteria.
- `session-wrapper-implementation-plan/testing-and-first-slice.md` owns testing strategy and first build slice.
- `session-wrapper-implementation-plan/decisions-before-coding.md` owns the historical pre-coding decisions.

## Rationale

The active backlog stays complete in `tasks.md`, while dense supporting planning material is moved behind a stable archive entrypoint. Agents can now load only the implementation-plan slice relevant to a question.

## Validation

- `rg -n "session-wrapper-implementation-plan/(objective-stack-repo|milestones|testing-and-first-slice|decisions-before-coding)|Session Wrapper Implementation Plan" workpads/research/archive/support/session-wrapper-implementation-plan.md workpads/research/archive/support/session-wrapper-implementation-plan`
- `wc -l workpads/research/archive/support/session-wrapper-implementation-plan.md workpads/research/archive/support/session-wrapper-implementation-plan/*.md`
- `git diff --check`
