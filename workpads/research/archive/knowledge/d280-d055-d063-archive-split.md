# D280: D55-D63 Archive Split

Split the dense owned-extractor foundation archive into smaller routed files without compacting or removing planned tasks.

Files:

- `workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation.md` is now the stable routing entrypoint.
- `workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation/d055-d057-static-markdown-foundation.md` preserves the first owned static extractor, Rust transport/CSS parsing, and initial markdown renderer decisions.
- `workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation/d058-d060-browser-fallback-and-cdp.md` preserves owned fallback extraction and first CDP renderer decisions.
- `workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation/d061-d063-render-retry-table.md` preserves primary rendered extraction, CSS-wait retry, and markdown table decisions.

The active task backlog remains intact in `workpads/research/tasks.md`; this is only a support-file routing split.

Validation:

- `diff -u <(git show HEAD:workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation.md) <(cat workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation/d055-d057-static-markdown-foundation.md workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation/d058-d060-browser-fallback-and-cdp.md workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation/d061-d063-render-retry-table.md)`
- `wc -l workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation.md workpads/research/archive/knowledge/d055-d063-owned-extractor-foundation/*.md workpads/research/tasks.md`
- `git diff --check`
