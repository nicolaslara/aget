# D272: D77-D91 Owned Extractor/Browser Archive Split

## Decision

Keep `workpads/research/archive/knowledge/d077-d091-owned-extractor-options-cleanup.md` as the stable routing entrypoint, but move the dense historical D77-D91 decision detail into smaller section files:

- `d077-d091-owned-extractor-options-cleanup/d077-d082-options-render-readiness.md`
- `d077-d091-owned-extractor-options-cleanup/d083-d085-browser-state-fallback.md`
- `d077-d091-owned-extractor-options-cleanup/d086-d091-cleanup-markdown.md`

This preserves `workpads/research/tasks.md` as the full planned-task source of truth. The task file was not compacted; it only records the completed support-file split.

## Boundary

The split is documentation-only. It does not change active migration scope, source-backed decisions, runtime behavior, or verification expectations.

## Validation

- `wc -l workpads/research/archive/knowledge/d077-d091-owned-extractor-options-cleanup.md workpads/research/archive/knowledge/d077-d091-owned-extractor-options-cleanup/*.md workpads/research/archive/knowledge/d272-d077-d091-archive-split.md workpads/research/tasks.md`
- `git diff --check`
