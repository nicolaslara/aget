# D269: D92-D108 Markdown/Browser Archive Split

## Decision

Keep `workpads/research/archive/knowledge/d092-d108-markdown-browser-slices.md` as the stable routing entrypoint, but move the dense historical D92-D108 decision detail into smaller section files:

- `d092-d108-markdown-browser-slices/d092-d097-markdown-and-chrome-startup.md`
- `d092-d108-markdown-browser-slices/d098-d103-inline-markdown-and-cdp-discovery.md`
- `d092-d108-markdown-browser-slices/d104-d108-lists-blockquotes-link-escaping.md`

This preserves `workpads/research/tasks.md` as the full planned-task source of truth. The task file was not compacted; it only records the completed support-file split.

## Boundary

The split is documentation-only. It does not change active migration scope, source-backed decisions, runtime behavior, or verification expectations.

## Validation

- `wc -l workpads/research/archive/knowledge/d092-d108-markdown-browser-slices.md workpads/research/archive/knowledge/d092-d108-markdown-browser-slices/*.md workpads/research/archive/knowledge/d269-d092-d108-archive-split.md workpads/research/tasks.md`
- `git diff --check`
