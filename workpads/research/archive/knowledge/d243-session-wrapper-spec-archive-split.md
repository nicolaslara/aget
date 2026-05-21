# D243: Session Wrapper Spec Archive Split

Date: 2026-05-22

Decision:

- Keep `workpads/research/tasks.md` intact as the executable backlog.
- Replace the oversized historical `workpads/research/archive/support/session-wrapper-poc-spec.md` body with a routing index.
- Move the preserved historical content into section files under `workpads/research/archive/support/session-wrapper-poc-spec/`:
  - `product-spec.md`
  - `v1-implementation-spec.md`
  - `verification-and-agent-contract.md`
  - `future-owned-direction.md`

Boundary:

- This is a mechanical support-file compaction only. It does not change current product direction, implementation code, or the active task backlog.
- The old archive path remains the entrypoint for discoverability.

Validation:

- `diff -u <(git show HEAD:workpads/research/archive/support/session-wrapper-poc-spec.md) <(cat workpads/research/archive/support/session-wrapper-poc-spec/product-spec.md workpads/research/archive/support/session-wrapper-poc-spec/v1-implementation-spec.md workpads/research/archive/support/session-wrapper-poc-spec/verification-and-agent-contract.md workpads/research/archive/support/session-wrapper-poc-spec/future-owned-direction.md)`
- `wc -l workpads/research/archive/support/session-wrapper-poc-spec.md workpads/research/archive/support/session-wrapper-poc-spec/*.md workpads/research/knowledge.md workpads/research/tasks.md`
- `git diff --check`
