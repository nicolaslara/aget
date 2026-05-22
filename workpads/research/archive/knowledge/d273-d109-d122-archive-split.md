# D273: D109-D122 Selector/Overlay/Shadow Archive Split

## Decision

Keep `workpads/research/archive/knowledge/d109-d122-selector-overlay-shadow.md` as the stable routing entrypoint, but move the dense historical D109-D122 decision detail into smaller section files:

- `d109-d122-selector-overlay-shadow/d109-d115-markdown-selector-escaping.md`
- `d109-d122-selector-overlay-shadow/d116-d120-chrome-overlay-cleanup.md`
- `d109-d122-selector-overlay-shadow/d121-d122-linked-images-shadow-dom.md`

This preserves `workpads/research/tasks.md` as the full planned-task source of truth. The task file was not compacted; it only records the completed support-file split.

## Boundary

The split is documentation-only. It does not change active migration scope, source-backed decisions, runtime behavior, or verification expectations.

## Validation

- `wc -l workpads/research/archive/knowledge/d109-d122-selector-overlay-shadow.md workpads/research/archive/knowledge/d109-d122-selector-overlay-shadow/*.md workpads/research/archive/knowledge/d273-d109-d122-archive-split.md workpads/research/tasks.md`
- `git diff --check`
