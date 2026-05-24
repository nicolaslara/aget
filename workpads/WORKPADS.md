# Active Workpads

Source of truth for active projects.

## Current Focus

| Project | Status | Description |
| --- | --- | --- |
| `post-migration` | 🔨 Active | CLI-first productization after the owned-backend migration. |
| `research` | 📚 Historical | Archived research, migration history, and source-reference evidence. |

## Project Context

### post-migration

Load:

```text
project.md
WORKING.md
workpads/post-migration/tasks.md
workpads/post-migration/knowledge.md
workpads/post-migration/references.md
```

Quick nav:

- `workpads/post-migration/tasks.md` §Phase 1 for documentation and source-of-truth cleanup
- `workpads/post-migration/tasks.md` §Phase 2 for historical dependency-surface removal
- `workpads/post-migration/tasks.md` §Phase 3 for upstream-inspired parity coverage
- `workpads/post-migration/tasks.md` §Phase 4 for diagnostics, release, artifacts, and bounded crawl/map/batch CLI work

### research

Load only when older design evidence, migration history, or source-project
research is needed:

```text
workpads/research/tasks.md
workpads/research/knowledge.md
workpads/research/references.md
```

Historical reference repos:

```bash
mkdir -p references/repos
git clone https://github.com/wevm/curl.md.git references/repos/curl-md
git clone https://github.com/firecrawl/firecrawl.git references/repos/firecrawl
```

Rules:

- Treat `post-migration` as the executable source of truth.
- Keep `aget` CLI-first; MCP is out of scope.
- Record primary-source links and source snapshots in the active `references.md`.
- Record decisions, risks, and rejected alternatives in the active `knowledge.md`.
