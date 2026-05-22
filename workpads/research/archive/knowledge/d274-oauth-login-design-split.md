# D274: OAuth-Safe Browser Login Design Split

## Decision

Keep `workpads/research/oauth-safe-browser-login-design.md` as the stable routing entrypoint, but move the dense design detail into smaller section files:

- `oauth-safe-browser-login-design/vocabulary-flow.md`
- `oauth-safe-browser-login-design/browser-errors-tests.md`

This preserves `workpads/research/tasks.md` as the full planned-task source of truth. The task file was not compacted; it only records the completed support-file split.

## Boundary

The split is documentation-only. It preserves the existing design path for I20/I21/I22 references and does not change auth/session behavior or browser-family scope.

## Validation

- `wc -l workpads/research/oauth-safe-browser-login-design.md workpads/research/oauth-safe-browser-login-design/*.md workpads/research/archive/knowledge/d274-oauth-login-design-split.md workpads/research/tasks.md`
- `git diff --check`
