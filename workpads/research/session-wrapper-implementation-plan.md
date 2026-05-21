# aget Session Wrapper Implementation Plan

This file is now a routing stub. The full historical implementation plan lives at:

- `workpads/research/archive/support/session-wrapper-implementation-plan.md`

Use the archived plan only when checking early milestone intent for the thin-wrapper PoC. For current implementation work, prefer:

- `workpads/research/tasks.md` for the live backlog and acceptance criteria.
- `workpads/research/knowledge.md` for current architecture decisions and open migration gaps.
- `workpads/research/references.md` for current source snapshots, code boundaries, and deeper archive routing.

The old plan described an external-tool-backed v1 wrapper. The active implementation has since migrated toward owned Rust backends while keeping explicit compatibility adapters for Crawl4AI and `agent-browser`.
