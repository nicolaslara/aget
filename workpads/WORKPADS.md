# Active Workpads

Source of truth for active projects.

## Current Focus

| Project | Status | Description |
| --- | --- | --- |
| `research` | 🔨 Active | Background research and technical direction for local auth-aware agent web fetching. |

## Project Context

### research

Load:

```text
project.md
WORKING.md
workpads/research/tasks.md
workpads/research/knowledge.md
workpads/research/references.md
```

Quick nav:

- `project.md` §Features To Preserve From curl.md
- `project.md` §Features To Preserve From Firecrawl
- `project.md` §New Local/Auth Requirements
- `project.md` §Rust Research Areas
- `workpads/research/tasks.md` §Phase 1 for source-project research
- `workpads/research/tasks.md` §Phase 2 for local browser/auth research
- `workpads/research/tasks.md` §Phase 3 for Rust feasibility

Reference repos to clone on demand:

```bash
mkdir -p references/repos
git clone https://github.com/wevm/curl.md.git references/repos/curl-md
git clone https://github.com/firecrawl/firecrawl.git references/repos/firecrawl
```

Rules:

- Do not begin implementation until research tasks identify a minimal architecture.
- Record primary-source links in `references.md`.
- Record decisions, risks, and rejected alternatives in `knowledge.md`.
