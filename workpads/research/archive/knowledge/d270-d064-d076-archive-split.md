# D270: D64-D76 Browser/Default-Switch Archive Split

## Decision

Keep `workpads/research/archive/knowledge/d064-d076-browser-default-switch.md` as the stable routing entrypoint, but move the dense historical D64-D76 decision detail into smaller section files:

- `d064-d076-browser-default-switch/d064-d068-chrome-import-login.md`
- `d064-d076-browser-default-switch/d069-d073-extraction-options-rendering.md`
- `d064-d076-browser-default-switch/d074-d076-default-switch-audit.md`

This preserves `workpads/research/tasks.md` as the full planned-task source of truth. The task file was not compacted; it only records the completed support-file split.

## Boundary

The split is documentation-only. It does not change active migration scope, source-backed decisions, runtime behavior, or verification expectations.

## Validation

- `wc -l workpads/research/archive/knowledge/d064-d076-browser-default-switch.md workpads/research/archive/knowledge/d064-d076-browser-default-switch/*.md workpads/research/archive/knowledge/d270-d064-d076-archive-split.md workpads/research/tasks.md`
- `git diff --check`
